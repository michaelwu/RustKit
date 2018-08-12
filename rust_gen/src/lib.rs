// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate clang_sys as clang;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

mod walker;

use walker::{CursorKind, TypeKind};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use quote::ToTokens;
use proc_macro2::{Ident, Span};

fn cursor_dump(c: &walker::Cursor, p: Option<&str>) {
    let mut prefix = "  ".to_owned();
    if let Some(p) = p {
        prefix.push_str(p);
    } else {
        prefix = "- ".to_owned();
    }

    c.visit_children(|c| {
        println!("{}Found {:?} with name \"{}\" ty kind \"{:?}\" {:?}", prefix, c.kind(), c.name(), c.ty().kind(), c.availability_attrs());
        cursor_dump(&c, Some(&prefix));
        walker::ChildVisit::Continue
    });
}

#[derive(Debug, PartialEq)]
enum Type {
    Void,
    Bool,
    Int(bool, usize),
    Long(bool),
    Float(usize),
    Pointer(Box<Type>, bool, bool),
    Record(String, bool),
    Enum(String),
    FunctionProto(Vec<Type>, Box<Type>, bool),
    FixedArray(Box<Type>, u64),
    Typedef(String),
    InstanceType(bool),
    SelectorRef,
    Id(Option<String>),
    Class(String, Vec<Type>, Vec<String>),
}

impl Type {
    pub fn read(t: &walker::Ty, name: Option<String>, nonnull: bool) -> Type {
        match t.kind() {
            TypeKind::Void => Type::Void,
            TypeKind::Bool => Type::Bool,
            TypeKind::SChar | TypeKind::Char_S => Type::Int(true, 1),
            TypeKind::UChar | TypeKind::Char_U => Type::Int(false, 1),
            TypeKind::Short => Type::Int(true, 2),
            TypeKind::UShort => Type::Int(false, 2),
            TypeKind::Int => Type::Int(true, 4),
            TypeKind::UInt => Type::Int(false, 4),
            TypeKind::Long => Type::Long(true),
            TypeKind::ULong => Type::Long(false),
            TypeKind::LongLong => Type::Int(true, 8),
            TypeKind::ULongLong => Type::Int(false, 8),
            TypeKind::Float => Type::Float(4),
            TypeKind::Double => Type::Float(8),
            TypeKind::Record => {
                let decl = t.decl();
                Type::Record(name.unwrap_or(decl.name()), decl.kind() == CursorKind::UnionDecl)
            },
            TypeKind::Enum => Type::Enum(name.unwrap_or(t.decl().name())),
            TypeKind::ConstantArray =>
                Type::FixedArray(
                    Box::new(Type::read(&t.element_ty(), None, false)),
                    t.array_size()),
            TypeKind::IncompleteArray =>
                Type::Pointer(
                    Box::new(Type::read(&t.element_ty(), None, false)),
                    nonnull,
                    false),
            TypeKind::Typedef => {
                let name = t.typedef_name();
                match name.as_str() {
                    "instancetype" =>
                        Type::Pointer(
                            Box::new(Type::InstanceType(nonnull)),
                            nonnull,
                            false),
                    "BOOL" => Type::Bool,
                    _ => {
                        let inner =
                            Type::read(
                                &t.decl().typedef_ty(),
                                Some(name.clone()),
                                nonnull
                            );
                        if inner.is_anonymous() {
                            Type::Typedef(name)
                        } else {
                            inner
                        }
                    }
                }
            },
            TypeKind::Attributed => {
                let n = t.nullability();
                Type::read(&t.modified_ty(), name, n == walker::Nullability::NonNull)
            },
            TypeKind::Elaborated => {
                Type::read(&t.named_type().unwrap(), name, nonnull)
            },
            TypeKind::Pointer => {
                Type::Pointer(Box::new(Type::read(&t.pointee(), None, false)), nonnull, t.is_const())
            },
            TypeKind::FunctionProto => {
                let args =
                    t.function_arg_iter().
                    map(|a| Type::read(&a, None, false)).collect();
                Type::FunctionProto(args, Box::new(Type::read(&t.result_type(), None, false)), t.is_variadic())
            },
            TypeKind::ObjCObjectPointer => {
                Type::Pointer(Box::new(Type::read(&t.pointee(), None, false)), nonnull, false)
            },
            TypeKind::ObjCSel => Type::SelectorRef,
            TypeKind::ObjCInterface => Type::Class(t.spelling(), Vec::new(), Vec::new()),
            TypeKind::ObjCId => Type::Pointer(Box::new(Type::Id(None)), nonnull, false),
            TypeKind::ObjCClass => Type::Pointer(Box::new(Type::Class("Class".to_owned(), Vec::new(), Vec::new())), nonnull, false),
            TypeKind::ObjCObject => {
                let bt = t.base_type().unwrap();
                match bt.kind() {
                    TypeKind::ObjCId => {
                        let proto =
                            t.protocol_ref_iter().map(|d| d.name()).next();
                        Type::Id(proto)
                    },
                    TypeKind::ObjCInterface | TypeKind::ObjCClass => {
                        let typeargs =
                            t.type_arg_iter().map(|t| Type::read(&t, None, false)).collect();
                        let proto: Vec<_> =
                            t.protocol_ref_iter().map(|d| d.name()).collect();
                        Type::Class(bt.spelling(), typeargs, proto)
                    },
                    _ => panic!("Unexpected base type kind {:?}", bt.kind()),
                }
            }
            _ => {
                println!("Unhandled type named {} with type kind {:?}", t.spelling(), t.kind());
                Type::Void
            },
        }
    }

    pub fn raw_ty(&self) -> syn::Type {
        match self {
            Type::Void => parse_quote!{ () },
            Type::Bool => parse_quote!{ bool },
            Type::Int(true, 1) => parse_quote!{ i8 },
            Type::Int(true, 2) => parse_quote!{ i16 },
            Type::Int(true, 4) => parse_quote!{ i32 },
            Type::Int(true, 8) => parse_quote!{ i64 },
            Type::Int(false, 1) => parse_quote!{ u8 },
            Type::Int(false, 2) => parse_quote!{ u16 },
            Type::Int(false, 4) => parse_quote!{ u32 },
            Type::Int(false, 8) => parse_quote!{ u64 },
            Type::Long(true) => parse_quote!{ isize },
            Type::Long(false) => parse_quote!{ usize },
            Type::Float(4) => parse_quote!{ f32 },
            Type::Float(8) => parse_quote!{ f64 },
            Type::FixedArray(inner, len) => {
                let inner_ty = inner.raw_ty();
                let array_len =
                    syn::LitInt::new(*len,
                                     syn::IntSuffix::None, Span::call_site());
                parse_quote!{ [#inner_ty; #array_len] }
            },
            Type::Pointer(inner, _, c) => {
                let inner_ty = if let Type::Void = **inner {
                    parse_quote!{ c_void }
                } else {
                    inner.raw_ty()
                };
                if let Type::FunctionProto(..) = **inner {
                    inner_ty
                } else if *c {
                    parse_quote!{ *const #inner_ty }
                } else {
                    parse_quote!{ *mut #inner_ty }
                }
            },
            Type::FunctionProto(args, retty, var) => {
                let retty = retty.raw_ty();
                let args: Vec<syn::Type> =
                    args.iter().map(|arg| arg.raw_ty()).collect();
                let mut f = parse_quote!{ extern fn (#(#args),*) -> #retty };
                if let syn::Type::BareFn(syn::TypeBareFn { ref mut variadic, .. }) = f {
                    if *var {
                        *variadic = Some(syn::token::Dot3::new(Span::call_site()));
                    }
                } else {
                    panic!("Bare function not generated???");
                }
                f
            },
            Type::InstanceType(_) => parse_quote!{ Self },
            Type::SelectorRef => parse_quote!{ SelectorRef },
            Type::Id(_) => parse_quote!{ Object },
            Type::Typedef(name) |
            Type::Enum(name) |
            Type::Record(name, ..) |
            Type::Class(name, ..) => {
                if name.is_empty() {
                    panic!("??? unnamed {:?}", self);
                }
                let path = Ident::new(&name, Span::call_site());
                parse_quote!{ #path }
            },
            _ => panic!("Unsupported type {:?}", self),
        }
    }

    pub fn rust_ty(&self, out: bool) -> syn::Type {
        match self {
            Type::Void => parse_quote!{ () },
            Type::Bool => parse_quote!{ bool },
            Type::Int(true, 1) => parse_quote!{ i8 },
            Type::Int(true, 2) => parse_quote!{ i16 },
            Type::Int(true, 4) => parse_quote!{ i32 },
            Type::Int(true, 8) => parse_quote!{ i64 },
            Type::Int(false, 1) => parse_quote!{ u8 },
            Type::Int(false, 2) => parse_quote!{ u16 },
            Type::Int(false, 4) => parse_quote!{ u32 },
            Type::Int(false, 8) => parse_quote!{ u64 },
            Type::Long(true) => parse_quote!{ isize },
            Type::Long(false) => parse_quote!{ usize },
            Type::Float(4) => parse_quote!{ f32 },
            Type::Float(8) => parse_quote!{ f64 },
            Type::FixedArray(inner, len) => {
                let inner_ty = inner.rust_ty(out);
                let array_len =
                    syn::LitInt::new(*len,
                                     syn::IntSuffix::None, Span::call_site());
                parse_quote!{ [#inner_ty; #array_len] }
            },
            Type::Pointer(inner, nonnull, c) => {
                if let Type::FunctionProto(..) = **inner {
                    return inner.raw_ty();
                }
                let inner_ty = if let Type::Void = **inner {
                    parse_quote!{ c_void }
                } else {
                    inner.rust_ty(true)
                };
                let inner_ty = if self.is_objc_object() {
                    if out {
                        parse_quote!{ Arc<#inner_ty> }
                    } else {
                        parse_quote!{ &#inner_ty }
                    }
                } else {
                    parse_quote!{ &#inner_ty }
                };
                let inner_ty = if let Type::Pointer(..) = **inner {
                    parse_quote!{ &mut #inner_ty }
                } else {
                    inner_ty
                };
                if *nonnull {
                    inner_ty
                } else {
                    parse_quote!{ Option<#inner_ty> }
                }
            },
            Type::InstanceType(_) => parse_quote!{ Self },
            Type::SelectorRef => parse_quote!{ SelectorRef },
            Type::Id(_) => parse_quote!{ Object },
            Type::Typedef(name) |
            Type::Enum(name) |
            Type::Record(name, false) |
            Type::Class(name, ..) => {
                let path = Ident::new(&name, Span::call_site());
                parse_quote!{ #path }
            },
            _ => panic!("Unsupported type {:?}", self),
        }
    }

    fn refs(&self, list: &mut Vec<String>) {
        match self {
            Type::FixedArray(inner, _) => inner.refs(list),
            Type::Pointer(inner, ..) => inner.refs(list),
            Type::Enum(name) |
            Type::Record(name, false) |
            Type::Id(Some(name)) => list.push(name.clone()),
            Type::Class(name, ta, pl) => {
                list.push(name.clone());
                for t in ta {
                    t.refs(list);
                }
                list.extend_from_slice(&pl);
            },
            Type::FunctionProto(args, retty, ..) => {
                for a in args {
                    a.refs(list);
                }
                retty.refs(list);
            },
            _ => (),
        }
    }

    pub fn is_objc_object(&self) -> bool {
        match self {
            Type::Pointer(inner, ..) => {
                match **inner {
                    Type::Id(..) |
                    Type::Class(..) |
                    Type::InstanceType(..) => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn is_anonymous(&self) -> bool {
        match self {
            Type::FixedArray(inner, ..) |
            Type::Pointer(inner, ..) => inner.is_anonymous(),
            Type::Enum(name) |
            Type::Record(name, ..) => name.is_empty(),
            _ => false,
        }
    }

    pub fn is_va_list(&self) -> bool {
        if let Type::FixedArray(inner, len) = self {
            if let Type::Record(ref name, false) = **inner {
                return name == "__va_list_tag";
            }
        }
        false
    }

    pub fn is_nonnull(&self) -> bool {
        match self {
            Type::Pointer(_, nonnull, _) => *nonnull,
            _ => unreachable!(),
        }
    }

    pub fn is_copy(&self) -> bool {
        match self {
            Type::Int(..) |
            Type::Long(..) |
            Type::Float(..) |
            Type::Enum(..) |
            Type::Bool => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Type::Int(signed, _) |
            Type::Long(signed) => *signed,
            Type::Float(..) => true,
            _ => false,
        }
    }

    pub fn to_raw_expr(&self, name: &str) -> syn::Expr {
        let mut temp_name = "__temp_".to_owned();
        temp_name.push_str(name);
        let temp_name = Ident::new(&temp_name, Span::call_site());
        let name = Ident::new(name, Span::call_site());
        match self {
            Type::Pointer(inner, nonnull, c) => {
                match **inner {
                    Type::Pointer(ref inner2, nonnull2, c2) => {
                        let nonnull_expr = parse_quote!{ &mut #temp_name as *mut _ };
                        if *nonnull {
                            nonnull_expr
                        } else {
                            parse_quote!{ #name.as_ref().map_or(ptr::null_mut(), |_| #nonnull_expr) }
                        }
                    },
                    _ => {
                        if *nonnull {
                            parse_quote!{ #name as *const _ as *mut _ }
                        } else {
                            parse_quote!{ #name.as_ref().map_or(ptr::null_mut(), |r| *r as *const _ as *mut _) }
                        }
                    }
                }
            }
            _ => parse_quote!{ #name }
        }
    }

    pub fn conversion_setup(&self, name: &str) -> Option<syn::Stmt> {
        match self {
            Type::Pointer(inner, ..) => {
                match **inner {
                    Type::Pointer(..) => {
                        let mut temp_name = "__temp_".to_owned();
                        temp_name.push_str(name);
                        let temp_name =
                            Ident::new(&temp_name, Span::call_site());
                        Some(parse_quote!{
                            let mut #temp_name = ptr::null_mut();
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn msg_send(&self) -> &'static str {
        match self {
            Type::Float(4) | Type::Float(8) => "objc_msgSend_fpret",
            _ => "objc_msgSend",
        }
    }
}

#[derive(Debug)]
struct PropertyDecl {
    ty: Type,
    getter: String,
    setter: Option<String>,
}

impl PropertyDecl {
    pub fn read(c: &walker::Cursor) -> PropertyDecl {
        let propname = c.name();
        let setter = if !c.property_attributes().readonly() {
            Some(c.setter_name())
        } else {
            None
        };
        PropertyDecl {
            ty: Type::read(&c.ty(), None, false),
            getter: c.getter_name(),
            setter: setter,
        }
    }
}

// List of reserved keywords in Rust that are not unusable in ObjC
fn is_reserved_keyword(s: &str) -> bool {
    match s {
        "as" |
        "create" |
        "false" |
        "fn" |
        "impl" |
        "let" |
        "loop" |
        "match" |
        "mod" |
        "mut" |
        "pub" |
        "ref" |
        "Self" |
        "self" |
        "super" |
        "trait" |
        "true" |
        "type" |
        "unsafe" |
        "use" |
        "where" |
        "abstract" |
        "alignof" |
        "become" |
        "box" |
        "final" |
        "macro" |
        "override" |
        "priv" |
        "proc" |
        "pure" |
        "self" |
        "virtual" |
        "yield" => true,
        _ => false,
    }
}

#[derive(Debug)]
struct Arg {
    name: String,
    ty: Type,
}

#[derive(Debug, PartialEq)]
enum ReturnOwnership {
    Retained,
    NotRetained,
    Autoreleased,
}

#[derive(Debug)]
struct MethodDecl {
    rustname: String,
    avail: walker::Availability,
    args: Vec<Arg>,
    retty: Type,
    ret_own: ReturnOwnership,
    inter_ptr: bool,
}

impl MethodDecl {
    pub fn read(c: &walker::Cursor) -> MethodDecl {
        let len = c.num_args();
        let fnty = c.ty();
        let args: Vec<_> =
            (0..len).map(|x| {
                let arg = c.arg(x);
                let mut name = arg.name();
                if is_reserved_keyword(&name) {
                    name.push('_');
                }
                Arg {
                    name: name,
                    ty: Type::read(&arg.ty(), None, false),
                }
            }).collect();
        let mut ownership = ReturnOwnership::Autoreleased;
        let mut inter_ptr = false;
        c.visit_children(|c| {
            match c.kind() {
                CursorKind::NSReturnsRetained =>
                    ownership = ReturnOwnership::Retained,
                CursorKind::NSReturnsNotRetained =>
                    ownership = ReturnOwnership::NotRetained,
                CursorKind::NSReturnsAutoreleased =>
                    ownership = ReturnOwnership::Autoreleased,
                CursorKind::ObjCReturnsInnerPointer =>
                    inter_ptr = true,
                _ => (),
            }
            walker::ChildVisit::Continue
        });
        let mut avail = c.availability();
        if let walker::Availability::Available = avail {
            let attrs = c.availability_attrs();
            let swift_attr = attrs.iter().find(|a| a.platform == "swift" && a.unavailable);
            if let Some(attr) = swift_attr {
                avail = walker::Availability::NotAvailable(attr.message.clone());
            }
        }
        let mut rustname = c.name().replace(":", "_");
        if is_reserved_keyword(&rustname) {
            rustname.push('_');
        }
        MethodDecl {
            rustname: rustname,
            avail: avail,
            args: args,
            retty: Type::read(&c.result_ty(), None, false),
            ret_own: ownership,
            inter_ptr: inter_ptr,
        }
    }
    pub fn refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        for a in &self.args {
            a.ty.refs(&mut refs);
        }
        self.retty.refs(&mut refs);
        refs
    }
}

#[derive(Debug)]
struct ClassDecl {
    src: PathBuf,
    rustname: String,
    superclass: String,
    protocols: Vec<String>,
    cprops: HashMap<String, PropertyDecl>,
    iprops: HashMap<String, PropertyDecl>,
    cmethods: HashMap<String, MethodDecl>,
    imethods: HashMap<String, MethodDecl>,
}

impl ClassDecl {
    pub fn read(c: &walker::Cursor) -> ClassDecl {
        println!("{}", c.name());
        let mut superclass = String::new();
        let mut protocols = Vec::new();
        let mut cprops = HashMap::new();
        let mut iprops: HashMap<String, PropertyDecl> = HashMap::new();
        let mut cmethods = HashMap::new();
        let mut imethods = HashMap::new();
        c.visit_children(|c| {
            match c.kind() {
                CursorKind::UnexposedAttr => {
                    println!("Found unexposed attr {}", c.name());
                }
                CursorKind::ObjCSuperClassRef => {
                    superclass = c.name();
                }
                CursorKind::ObjCProtocolRef => {
                    protocols.push(c.name());
                }
                CursorKind::ObjCClassMethodDecl => {
                    let old = cmethods.insert(c.name(), MethodDecl::read(&c));
                    if old.is_some() {
                        panic!("????");
                    }
                }
                CursorKind::ObjCInstanceMethodDecl => {
                    let selname = c.name();
                    if iprops.values().
                        any(|p|
                            &p.getter == &selname ||
                            p.setter.as_ref() == Some(&selname)) {
                        return walker::ChildVisit::Continue
                    }
                    let old = imethods.insert(selname, MethodDecl::read(&c));
                    if old.is_some() {
                        panic!("????");
                    }
                }
                CursorKind::ObjCPropertyDecl => {
                    let classprop = c.property_attributes().class();
                    let decl = PropertyDecl::read(&c);
                    if classprop {
                        let old = cprops.insert(c.name(), decl);
                        if old.is_some() {
                            panic!("Duplicate class property declaration");
                        }
                    } else {
                        let old = iprops.insert(c.name(), decl);
                        if old.is_some() {
                            panic!("Duplicate property declaration");
                        }
                    }
                }
                CursorKind::ObjCClassRef => {
                    // Same as ObjCSuperClassRef, right?
                }
                _ => {
                    println!("Unknown cursor kind {:?}", c.kind());
                }
            };
            return walker::ChildVisit::Continue;
        });
        ClassDecl {
            src: c.location().filename(),
            rustname: c.name(),
            superclass: superclass,
            protocols: protocols,
            cprops: cprops,
            iprops: iprops,
            cmethods: cmethods,
            imethods: imethods,
        }
    }

    pub fn collect_selectors(&self, h: &mut HashSet<String>) {
        for p in self.iprops.values() {
            h.insert(p.getter.clone());
            if let Some(ref setter) = p.setter {
                h.insert(setter.to_owned());
            }
        }
        for m in self.cmethods.keys() {
            h.insert(m.clone());
        }
        for m in self.imethods.keys() {
            h.insert(m.clone());
        }
    }
}

#[derive(Debug)]
struct EnumDecl {
    src: PathBuf,
    rustname: String,
    ty: Type,
    exhaustive: bool,
    flagenum: bool,
    variants: Vec<(String, u64, bool)>,
}

impl EnumDecl {
    pub fn read(c: &walker::Cursor) -> EnumDecl {
        let mut variants = Vec::new();
        let ty = Type::read(&c.enum_ty(), None, false);
        let mut flagenum = false;
        c.visit_children(|c| {
            match c.kind() {
                CursorKind::EnumConstantDecl => {
                    let (val, neg) = if ty.is_signed() {
                        let val = c.enum_const_value_signed();
                        let neg = val < 0;
                        let val =
                            val.checked_abs().
                            map_or(i64::max_value() as u64 + 1,
                                   |v| v as u64);
                        (val, neg)
                    } else {
                        let val = c.enum_const_value_unsigned();
                        (val, false)
                    };

                    if variants.iter().
                        any(|(_, v, s)| *v == val && *s == neg) {
                        println!("Skipping {} due to duplicated value", c.name());
                        return walker::ChildVisit::Continue;
                    }
                    variants.push((
                        c.name(),
                        val,
                        neg
                    ));
                },
                CursorKind::FlagEnum => {
                    flagenum = true;
                },
                _ => (),
            }
            walker::ChildVisit::Continue
        });
        EnumDecl {
            src: c.location().filename(),
            rustname: c.name(),
            ty: ty,
            exhaustive: false,
            flagenum: flagenum,
            variants: variants,
        }
    }
}

#[derive(Debug)]
struct RecordDecl {
    src: PathBuf,
    rustname: String,
    fields: Vec<(String, Type)>,
    union: bool,
}

impl RecordDecl {
    pub fn read(c: &walker::Cursor) -> Vec<RecordDecl> {
        let mut fields = Vec::new();
        let struct_name = c.name();
        let mut res = Vec::new();
        c.visit_children(|c| {
            match c.kind() {
                CursorKind::FieldDecl => {
                    let name = c.name();
                    if name.is_empty() {
                        println!("Skipping unnamed field in {}", struct_name);
                        return walker::ChildVisit::Continue;
                    }
                    let ty = Type::read(&c.ty(), None, false);
                    if let Type::Record(ref name, ..) = ty {
                        if name.is_empty() {
                            println!("Skipping field to anon record in {}.{}", struct_name, name);
                            return walker::ChildVisit::Continue;
                        }
                    }
                    fields.push((name, ty));
                }
                CursorKind::StructDecl | CursorKind::UnionDecl => {
                    let name = c.name();
                    if name.is_empty() {
                        println!("Skipping anon record decl in {}", struct_name);
                        return walker::ChildVisit::Continue;
                    }
                    res.append(&mut RecordDecl::read(&c));
                }
                _ => ()
            }
            walker::ChildVisit::Continue
        });
        res.push(RecordDecl {
            src: c.location().filename(),
            rustname: struct_name,
            fields: fields,
            union: c.kind() == CursorKind::UnionDecl,
        });
        res
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        for (_, t) in &self.fields {
            t.refs(&mut refs);
        }
        refs
    }
}

#[derive(Debug)]
struct TypedefDecl {
    src: PathBuf,
    rustname: String,
    ty: Type,
}

impl TypedefDecl {
    pub fn read(c: &walker::Cursor) -> TypedefDecl {
        TypedefDecl {
            src: c.location().filename(),
            rustname: c.name(),
            ty: Type::read(&c.typedef_ty(), None, false),
        }
    }
    pub fn refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        self.ty.refs(&mut refs);
        refs
    }
}

#[derive(Debug)]
struct FunctionDecl {
    src: PathBuf,
    rustname: String,
    avail: walker::Availability,
    args: Vec<(String, Type)>,
    retty: Type,
    variadic: bool,
}

impl FunctionDecl {
    pub fn read(c: &walker::Cursor) -> FunctionDecl {
        let args =
            c.arg_iter().map(|a|
                (a.name(), Type::read(&a.ty(), None, false))
            ).collect();
        let mut avail = c.availability();
        if let walker::Availability::Available = avail {
            let attrs = c.availability_attrs();
            let swift_attr = attrs.iter().find(|a| a.platform == "swift" && a.unavailable);
            if let Some(attr) = swift_attr {
                avail = walker::Availability::NotAvailable(attr.message.clone());
            }
        }
        FunctionDecl {
            src: c.location().filename(),
            rustname: c.spelling(),
            avail: avail,
            args: args,
            retty: Type::read(&c.result_ty(), None, false),
            variadic: c.is_variadic(),
        }
    }
    pub fn refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        for (_, ty) in &self.args {
            ty.refs(&mut refs);
        }
        self.retty.refs(&mut refs);
        refs
    }
}

#[derive(Debug)]
enum ItemDecl {
    Enum(EnumDecl),
    Record(RecordDecl),
    Class(ClassDecl),
    Typedef(TypedefDecl),
    Func(FunctionDecl),
}

impl ItemDecl {
    fn src(&self) -> &Path {
        match self {
            ItemDecl::Enum(e) => &e.src,
            ItemDecl::Record(s) => &s.src,
            ItemDecl::Class(c) => &c.src,
            ItemDecl::Typedef(t) => &t.src,
            ItemDecl::Func(f) => &f.src,
        }
    }
    fn framework_name(&self) -> Vec<String> {
        let mut names = Vec::new();
        let src = self.src();
        let mut p = src.parent();
        while let Some(parent) = p {
            if let Some(ext) = parent.extension() {
                if "framework" == ext {
                    names.push(parent.file_stem().unwrap().to_str().unwrap().to_owned());
                }
            }
            p = parent.parent();
        }
        names
    }
}

pub fn bind_framework(
    sdk_path: &Path,
    framework_name: &str,
    out_dir: &Path,
) -> HashSet<String> {
    if !clang::is_loaded() {
        clang::load().unwrap();
    }

    let mut framework_path = sdk_path.to_owned();
    framework_path.push("System/Library/Frameworks");
    framework_path.push(&format!("{}.framework/Headers", framework_name));
    let mut include_path = framework_path.clone();
    include_path.push(&format!("{}.h", framework_name));
    let sdk_path_str = sdk_path.to_str().unwrap();
    let idx = walker::Index::new().unwrap();
    let tu =
        idx.parse_tu(&[
            "-ObjC",
            "-fobjc-arc",
            "-fno-objc-exceptions",
            "-fobjc-abi-version=2",
            &format!("-F{}/System/Library/Frameworks", sdk_path_str),
            &format!("-I{}/usr/include", sdk_path_str),
        ], &include_path).unwrap();
    let mut out_path = out_dir.to_owned();
    out_path.push(&format!("{}.rs", framework_name));
    bind_tu(&tu, &framework_path, Some(framework_name), &out_path)
}

pub fn bind_file(
    sdk_path: &Path,
    header_path: &Path,
    out_dir: &Path,
) {
    if !clang::is_loaded() {
        clang::load().unwrap();
    }

    let sdk_path_str = sdk_path.to_str().unwrap();
    let idx = walker::Index::new().unwrap();
    let tu =
        idx.parse_tu(&[
            "-ObjC",
            "-fobjc-arc",
            "-fno-objc-exceptions",
            "-fobjc-abi-version=2",
            &format!("-F{}/System/Library/Frameworks", sdk_path_str),
            &format!("-I{}/usr/include", sdk_path_str),
        ], &header_path).unwrap();
    let mut out_path = out_dir.to_owned();
    out_path.push(&format!("{}.rs", header_path.file_stem().unwrap().to_str().unwrap()));
    bind_tu(&tu, &header_path, None, &out_path);
}

pub fn bind_tu(
    tu: &walker::TranslationUnit,
    base_path: &Path,
    framework_name: Option<&str>,
    out_path: &Path,
) -> HashSet<String> {
    let mut decls = HashMap::new();
    let mut declnames = Vec::new();
    let mut anonnames = Vec::new();
    tu.visit(|c| {
        match c.kind() {
            CursorKind::ObjCInterfaceDecl => {
                let name = c.name();
                let class = ClassDecl::read(&c);
                if c.location().filename().starts_with(base_path) {
                    println!("{:#?}", class);
                    cursor_dump(&c, None);
                }
                let old = decls.insert(name.clone(), ItemDecl::Class(class));
                if old.is_some() {
                    panic!("??? class {} already defined", c.name());
                }
                declnames.push(name);
            },
            CursorKind::EnumDecl => {
                let name = c.name();
                if name.is_empty() {
                    println!("Skipping anonymous enum");
                    cursor_dump(&c, None);
                    return walker::ChildVisit::Continue;
                }
                if !c.is_definition() {
                    return walker::ChildVisit::Continue;
                }
                let decl = EnumDecl::read(&c);
                if c.location().filename().starts_with(base_path) {
                    println!("{:#?}", decl);
                    cursor_dump(&c, None);
                }
                let old = decls.insert(name.clone(), ItemDecl::Enum(decl));
                if old.is_some() {
                    panic!("??? enum {} already defined", name);
                }
                declnames.push(name);
            },
            CursorKind::StructDecl | CursorKind::UnionDecl => {
                let name = c.name();
                if name.is_empty() {
                    println!("Skipping anonymous record");
                    cursor_dump(&c, None);
                    return walker::ChildVisit::Continue;
                }
                if c.is_definition() && decls.contains_key(&name) {
                    return walker::ChildVisit::Continue;
                }
                let decl = RecordDecl::read(&c);
                if c.location().filename().starts_with(base_path) {
                    for d in &decl {
                        println!("{:#?}", d);
                    }
                    cursor_dump(&c, None);
                }
                for d in decl {
                    let declname = d.rustname.clone();
                    let old = decls.insert(declname.clone(), ItemDecl::Record(d));
                    if let Some(old) = old {
                        if let ItemDecl::Record(old) = old {
                            if !old.is_empty() {
                                println!("??? record {} already defined", declname);
                            }
                        } else {
                            panic!("Old definition not a record??? {} : {:?}", declname, old);
                        }
                    } else {
                        declnames.push(declname);
                    }
                }
            },
            CursorKind::TypedefDecl => {
                let ty = c.typedef_ty();
                let mut standard_typedef = false;
                match ty.kind() {
                    TypeKind::Elaborated => {
                        let nty = ty.named_type().unwrap();
                        let decl = nty.decl();
                        let mut decl_name = decl.name();
                        if decl_name.is_empty() {
                            decl_name.push_str(&c.name());
                        }
                        if nty.kind() == TypeKind::Record {
                            let decl = decls.entry(decl_name.clone()).or_insert_with(|| {
                                let mut r = RecordDecl::read(&decl).pop().unwrap();
                                r.rustname = decl_name.clone();
                                anonnames.push((ty.canonical().decl().location(), decl_name.clone()));
                                declnames.push(decl_name);
                                ItemDecl::Record(r)
                            });
                            if let ItemDecl::Record(s) = decl {
                                let name = c.name();
                                if s.src == c.location().filename() &&
                                   s.rustname.is_empty() {
                                    s.rustname = name.clone();
                                    anonnames.push((ty.canonical().decl().location(), name));
                                } else {
                                    standard_typedef = s.rustname != name;
                                }
                            } else {
                                panic!("Expected a RecordDecl, got {:?}", decl);
                            }
                        } else if nty.kind() == TypeKind::Enum {
                            if decls.contains_key(&decl_name) {
                                decls.entry(decl_name.clone()).and_modify(|i| {
                                    if let ItemDecl::Enum(ref mut e) = i {
                                        e.rustname = c.name();
                                    } else {
                                        panic!("Expected a EnumDecl, got {:?}", i);
                                    }
                                });
                            } else if decl.name().is_empty() {
                                let mut e = EnumDecl::read(&decl);
                                e.rustname = decl_name.clone();
                                declnames.push(decl_name.clone());
                                decls.insert(decl_name, ItemDecl::Enum(e));
                            }
                        } else {
                            println!("Not a record or enum. is a {:?}", nty.kind());
                        }
                    },
                    TypeKind::Typedef => {
                        standard_typedef = true;
                    },
                    TypeKind::Pointer => {
                        let pointee = ty.pointee();
                        let canonical = pointee.canonical();
                        if canonical.kind() == TypeKind::Record &&
                           canonical.decl().name().is_empty() {
                            let cdecl = canonical.decl();
                            let loc = cdecl.location();
                            let realname = anonnames.iter().find(|(l, _)| *l == loc);
                            if let Some((_, name)) = realname {
                                let mut decl = TypedefDecl::read(&c);
                                if let Type::Pointer(ref mut ty, ..) = decl.ty {
                                    if let Type::Record(_, u) = **ty {
                                        **ty = Type::Record(name.clone(), u);
                                    }
                                }
                                decls.insert(c.name(), ItemDecl::Typedef(decl));
                                declnames.push(c.name());
                            }
                        }
                    },
                    _ => {
                        println!("unhandled typedef pointing to {:?} named {}", ty.kind(), c.name());
                    }
                }
                if !standard_typedef {
                    return walker::ChildVisit::Continue;
                }
                let decl = TypedefDecl::read(&c);
                if c.location().filename().starts_with(base_path) {
                    println!("{:#?}", decl);
                    cursor_dump(&c, None);
                }
                let name = c.name();
                let old = decls.insert(name.clone(), ItemDecl::Typedef(decl));
                if old.is_some() {
                    println!("??? typedef {} already defined", name);
                } else {
                    declnames.push(name);
                }
            }
            CursorKind::FunctionDecl => {
                let decl = FunctionDecl::read(&c);
                if c.location().filename().starts_with(base_path) {
                    println!("{:#?}", decl);
                    cursor_dump(&c, None);
                }
                let spelling = c.spelling();
                let old = decls.insert(spelling.clone(), ItemDecl::Func(decl));
                if old.is_some() {
                    println!("??? function {} already defined", spelling);
                } else {
                    declnames.push(spelling);
                }
            }
            _ => (),
        };
        walker::ChildVisit::Continue
    });

    let mut subframeworks_path = base_path.to_owned();
    subframeworks_path.pop();
    subframeworks_path.push("Frameworks");
    let mods = std::fs::read_dir(&subframeworks_path).map(|rd| rd.map(|e| e.unwrap().path().file_stem().unwrap().to_str().unwrap().to_owned()).collect::<Vec<_>>()).unwrap_or(Vec::new());

    let mut deps = HashSet::new();
    if mods.is_empty() {
        gen_file(&decls, &declnames, base_path, &mods, framework_name, framework_name.is_none(), out_path, &mut deps);
        return deps;
    }

    let mut out_path = out_path.to_owned();
    out_path.pop();
    out_path.push(framework_name.unwrap());
    std::fs::create_dir(&out_path);
    {
        let mut subout_path = out_path.clone();
        subout_path.push("mod.rs");
        gen_file(&decls, &declnames, base_path, &mods, framework_name, false, &subout_path, &mut deps);
    }
    for m in mods {
        let mut subbase_path = subframeworks_path.to_owned();
        subbase_path.push(&format!("{}.framework/Headers", m));
        let mut subout_path = out_path.clone();
        subout_path.push(&format!("{}.rs", m));
        gen_file(&decls, &declnames, &subbase_path, &[], None, false, &subout_path, &mut deps);
    }
    deps
}

fn gen_file(
    decls: &HashMap<String, ItemDecl>,
    declnames: &[String],
    base_path: &Path,
    mods: &[String],
    framework_name: Option<&str>,
    file_mode: bool,
    out_path: &Path,
    deps: &mut HashSet<String>,
) {
    let mut selectors = HashSet::new();
    for d in decls.values() {
        if let ItemDecl::Class(c) = d {
            if c.src.starts_with(base_path) {
                c.collect_selectors(&mut selectors);
            }
        }
    }

    let mut uses = HashSet::new();
    for d in decls.values() {
        if !d.src().starts_with(base_path) {
            continue;
        }
        match d {
            ItemDecl::Enum(_) => {},
            ItemDecl::Record(s) => {
                for r in s.refs() {
                    uses.insert(r);
                }
            },
            ItemDecl::Class(c) => {
                uses.insert(c.superclass.clone());
                for p in &c.protocols {
                    uses.insert(p.clone());
                }
                for (_, m) in &c.cmethods {
                    for r in m.refs() {
                        uses.insert(r);
                    }
                }
                for (_, m) in &c.imethods {
                    for r in m.refs() {
                        uses.insert(r);
                    }
                }
            },
            ItemDecl::Typedef(t) => {
                for r in t.refs() {
                    uses.insert(r);
                }
            }
            ItemDecl::Func(f) => {
                for r in f.refs() {
                    uses.insert(r);
                }
            }
        }
    }
    let uses: Vec<_> = uses.iter().filter_map(|n| {
        match decls.get(n) {
            Some(d) => {
                if d.src().starts_with(base_path) {
                    None
                } else {
                    let name = d.framework_name();
                    let n = Ident::new(n, Span::call_site());
                    let mut path: syn::Path = parse_quote!{ #n };
                    for comp in &name {
                        let comp = Ident::new(comp, Span::call_site());
                        path = parse_quote!{ #comp::#path };
                    }
                    if let Some(comp) = name.iter().last() {
                        deps.insert(comp.to_owned());
                    }
                    Some(path)
                }
            }
            None => {
                if n == "NSString" {
                    Some(parse_quote!{ Foundation::NSString })
                } else {
                    None
                }
            }
        }
    }).collect();
    let mut ast = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: Vec::new(),
    };

    ast.items.push(parse_quote!{
        #[allow(unused_imports)]
        use objc::*;
    });
    if !file_mode {
        ast.items.push(parse_quote!{
            #[allow(unused_imports)]
            use std::ptr;
        });
        ast.items.push(parse_quote!{
            #[allow(unused_imports)]
            use std::mem;
        });
        ast.items.push(parse_quote!{
            #[allow(unused_imports)]
            use c_void;
        });
    }
    ast.items.extend(uses.iter().map(|p| {
        parse_quote!{
            use #p;
        }
    }));
    for m in mods {
        let m = Ident::new(&m, Span::call_site());
        ast.items.push(parse_quote!{
            pub mod #m;
        });
    }

    for s in selectors {
        let mut sel = s.as_bytes().to_owned();
        sel.push(0);
        let sel = proc_macro2::Literal::byte_string(&sel);
        let mut selname = "SEL_".to_owned();
        selname.push_str(&s.replace(":", "_"));
        let selname = Ident::new(&selname, Span::call_site());
        ast.items.push(parse_quote!{
            #[allow(non_upper_case_globals)]
            #[link_section="__DATA,__objc_selrefs"]
            pub static mut #selname: SelectorRef = SelectorRef(&#sel[0] as *const u8);
        });
    }

    for k in declnames {
        match decls.get(k).unwrap() {
            ItemDecl::Enum(e) => {
                if !e.src.starts_with(base_path) {
                    continue;
                }
                let variants: Vec<syn::Variant> = e.variants.iter().map(|(n, v, neg)| {
                    let var_name = Ident::new(&n, Span::call_site());
                    let var_val =
                        syn::LitInt::new(*v, syn::IntSuffix::None, Span::call_site());
                    if *neg {
                        parse_quote!{
                            #var_name = -#var_val
                        }
                    } else {
                        parse_quote!{
                            #var_name = #var_val
                        }
                    }
                }).collect();
                let enum_name = Ident::new(&e.rustname, Span::call_site());
                let repr_type = e.ty.rust_ty(false);
                if e.flagenum {
                    ast.items.push(parse_quote!{
                        bitflags! {
                            #[repr(C)]
                            pub struct #enum_name: #repr_type {
                                #(const #variants;)*
                            }
                        }
                    });
                } else {
                    ast.items.push(parse_quote!{
                        #[repr(#repr_type)]
                        #[derive(Copy, Clone)]
                        pub enum #enum_name {
                            #(#variants),*
                        }
                    });
                }
            }
            ItemDecl::Record(s) => {
                if !s.src.starts_with(base_path) {
                    continue;
                }
                let struct_name = Ident::new(&s.rustname, Span::call_site());
                let field_name: Vec<syn::Ident> = s.fields.iter().map(|(n, _)| {
                    let mut n = n.to_owned();
                    if is_reserved_keyword(&n) {
                        n.push('_');
                    }
                    Ident::new(&n, Span::call_site())
                }).collect();
                let field_ty: Vec<syn::Type> = s.fields.iter().map(|(_, t)| {
                    t.raw_ty()
                }).collect();

                if s.fields.is_empty() {
                    ast.items.push(parse_quote!{
                        #[repr(C)]
                        pub struct #struct_name {
                            opaque: u32,
                        }
                    });
                } else if s.union {
                    ast.items.push(parse_quote!{
                        #[repr(C)]
                        #[derive(Copy, Clone)]
                        pub union #struct_name {
                            #(pub #field_name : #field_ty),*
                        }
                    });
                } else {
                    ast.items.push(parse_quote!{
                        #[repr(C)]
                        #[derive(Copy, Clone)]
                        pub struct #struct_name {
                            #(pub #field_name : #field_ty),*
                        }
                    });
                }
            }
            ItemDecl::Typedef(t) => {
                if !t.src.starts_with(base_path) || t.ty.is_va_list() {
                    continue;
                }
                let name = Ident::new(&t.rustname, Span::call_site());
                let ty = t.ty.raw_ty();
                ast.items.push(parse_quote!{
                    pub type #name = #ty;
                });
            }
            ItemDecl::Class(c) => {
                if !c.src.starts_with(base_path) {
                    continue;
                }
                let mut class_rustname = k.clone();
                class_rustname.push_str("Class");
                let class_rustname =
                    Ident::new(&class_rustname, Span::call_site());
                let mut class_sym = "OBJC_CLASS_$_".to_owned();
                class_sym.push_str(&k);
                ast.items.push(parse_quote!{
                    extern {
                        #[link_name=#class_sym]
                        static #class_rustname: Class;
                    }
                });
                let mut classrefname = "CLASS_".to_owned();
                classrefname.push_str(&k);
                let classrefname = Ident::new(&classrefname, Span::call_site());
                ast.items.push(parse_quote!{
                    #[allow(non_upper_case_globals)]
                    #[link_section="__DATA,__objc_classrefs"]
                    static #classrefname: ClassRef = ClassRef(unsafe { &#class_rustname } as *const _);
                });
                let name =
                    Ident::new(&c.rustname, Span::call_site());
                ast.items.push(parse_quote!{
                    #[repr(C)]
                    pub struct #name {
                        isa: *const Class,
                    }
                });

                let mut methods: Vec<syn::ImplItem> = Vec::new();
                for (s, m) in &c.cmethods {
                    if let walker::Availability::NotAvailable(_) = m.avail {
                        continue;
                    }
                    if m.args.iter().any(|a| a.ty.is_va_list()) {
                        continue;
                    }
                    let mname =
                        Ident::new(&m.rustname, Span::call_site());
                    let mut selname = "SEL_".to_owned();
                    selname.push_str(&s.replace(":", "_"));
                    let selname =
                        Ident::new(&selname, Span::call_site());
                    let params: Vec<syn::FnArg> =
                        (&m.args).iter().
                        map(|a| {
                            let name = Ident::new(&a.name, Span::call_site());
                            let rawty = a.ty.rust_ty(false);
                            parse_quote!{ #name : #rawty }
                        }).collect();
                    let params = &params;
                    let rawtypes: Vec<_> =
                        (&m.args).iter().map(|a| a.ty.raw_ty()).collect();
                    let raw_ret_ty = m.retty.raw_ty();
                    let rust_ret_ty = m.retty.rust_ty(true);
                    let msgsend =
                        Ident::new(m.retty.msg_send(), Span::call_site());
                    let args: Vec<syn::Expr> =
                        (&m.args).iter().
                        map(|a| a.ty.to_raw_expr(&a.name)).collect();
                    let setup: Vec<_> =
                        (&m.args).iter().
                        filter_map(|a| a.ty.conversion_setup(&a.name)).collect();
                    let mut finish: Vec<syn::Stmt> = Vec::new();
                    if ReturnOwnership::Autoreleased == m.ret_own &&
                       m.retty.is_objc_object() {
                        finish.push(parse_quote!{
                            objc_retainAutoreleasedReturnValue(_ret as *mut _);
                        });
                    }
                    if m.retty.is_objc_object() {
                        if m.retty.is_nonnull() {
                            finish.push(parse_quote!{
                                let _ret = Arc::new_unchecked(_ret);
                            });
                        } else {
                            finish.push(parse_quote!{
                                let _ret = Arc::new(_ret);
                            });
                        }
                    }
                    methods.push(parse_quote!{
                        pub fn #mname(#(#params),*) -> #rust_ret_ty {
                            #(#setup)*
                            unsafe {
                                let send:
                                    unsafe extern "C" fn(
                                        *mut Class,
                                        SelectorRef,
                                        #(#rawtypes),*) -> #raw_ret_ty =
                                    mem::transmute(#msgsend as *const u8);
                                let _ret = send(
                                    #classrefname.0 as *const _ as *mut _,
                                    #selname,
                                    #(#args),*
                                );
                                #(#finish)*
                                _ret
                            }
                        }
                    });
                }
                for (s, m) in &c.imethods {
                    if let walker::Availability::NotAvailable(_) = m.avail {
                        continue;
                    }
                    if m.args.iter().any(|a| a.ty.is_va_list()) {
                        continue;
                    }
                    if c.cmethods.contains_key(s) {
                        continue;
                    }
                    let initializer = m.rustname.starts_with("init");
                    let mname = if initializer {
                        m.rustname.replacen("init", "new", 1)
                    } else {
                        m.rustname.clone()
                    };
                    let mname = Ident::new(&mname, Span::call_site());
                    let mut selname = "SEL_".to_owned();
                    selname.push_str(&s.replace(":", "_"));
                    let selname =
                        Ident::new(&selname, Span::call_site());
                    let mut params: Vec<syn::FnArg> =
                        (&m.args).iter().
                        map(|a| {
                            let name = Ident::new(&a.name, Span::call_site());
                            let rawty = a.ty.rust_ty(false);
                            parse_quote!{ #name : #rawty }
                        }).collect();
                    if !initializer {
                        params.insert(0, parse_quote!{ &self });
                    }
                    let params = &params;
                    let rawtypes: Vec<_> =
                        (&m.args).iter().map(|a| a.ty.raw_ty()).collect();
                    let raw_ret_ty = m.retty.raw_ty();
                    let rust_ret_ty = if m.retty.is_objc_object() || m.inter_ptr {
                        m.retty.rust_ty(true)
                    } else {
                        m.retty.raw_ty()
                    };
                    let msgsend =
                        Ident::new(m.retty.msg_send(), Span::call_site());
                    let args: Vec<syn::Expr> =
                        (&m.args).iter().
                        map(|a| a.ty.to_raw_expr(&a.name)).collect();
                    let setup: Vec<_> =
                        (&m.args).iter().
                        filter_map(|a| a.ty.conversion_setup(&a.name)).collect();
                    let mut finish: Vec<syn::Stmt> = Vec::new();
                    if ReturnOwnership::Autoreleased == m.ret_own &&
                       m.retty.is_objc_object() {
                        finish.push(parse_quote!{
                            objc_retainAutoreleasedReturnValue(_ret as *mut _);
                        });
                    }
                    if m.retty.is_objc_object() {
                        if m.retty.is_nonnull() {
                            finish.push(parse_quote!{
                                let _ret = Arc::new_unchecked(_ret);
                            });
                        } else {
                            finish.push(parse_quote!{
                                let _ret = Arc::new(_ret);
                            });
                        }
                    } else if m.inter_ptr {
                        if m.retty.is_nonnull() {
                            finish.push(parse_quote!{
                                let _ret = &*_ret;
                            });
                        } else {
                            finish.push(parse_quote!{
                                let _ret = if _ret.is_null() {
                                    None
                                } else {
                                    Some(&*_ret)
                                };
                            });
                        }
                    }
                    let get_obj: syn::Expr =
                        if initializer {
                            parse_quote!(objc_allocWithZone(#classrefname))
                        } else {
                            parse_quote!(self as *const Self as *mut Self as *mut _)
                        };
                    methods.push(parse_quote!{
                        pub fn #mname(#(#params),*) -> #rust_ret_ty {
                            #(#setup)*
                            unsafe {
                                let send:
                                    unsafe extern "C" fn(
                                        *mut Object,
                                        SelectorRef,
                                        #(#rawtypes),*) -> #raw_ret_ty =
                                    mem::transmute(#msgsend as *const u8);
                                let _ret = send(
                                    #get_obj,
                                    #selname,
                                    #(#args),*
                                );
                                #(#finish)*
                                _ret
                            }
                        }
                    });
                }

                ast.items.push(parse_quote!{
                    impl #name {
                        #(#methods)*
                    }
                });
            }
            ItemDecl::Func(_) => {}
        }
    }

    let funcs: Vec<syn::ForeignItem> = decls.values().filter_map(|i| {
        if let ItemDecl::Func(f) = i {
            if let walker::Availability::NotAvailable(_) = f.avail {
                None
            } else {
                Some(f)
            }
        } else {
            None
        }
    }).filter_map(|f| {
        if !f.src.starts_with(base_path) {
            return None;
        }
        if f.args.iter().any(|(_, t)| t.is_va_list()) {
            return None;
        }
        let name = Ident::new(&f.rustname, Span::call_site());
        let arg_name: Vec<Ident> =
            f.args.iter().map(|(n, _)| {
                let mut name = n.to_owned();
                if is_reserved_keyword(n) || n.is_empty() {
                    name.push('_');
                }
                Ident::new(&name, Span::call_site())
            }).collect();
        let arg_ty: Vec<syn::Type> =
            f.args.iter().map(|(_, t)| t.raw_ty()).collect();
        let retty = f.retty.raw_ty();
        let mut fndecl: syn::ForeignItemFn = parse_quote!{
            pub fn #name(#(#arg_name: #arg_ty),*) -> #retty;
        };
        if f.variadic {
            fndecl.decl.variadic = Some(syn::token::Dot3::new(Span::call_site()));
        }
        Some(syn::ForeignItem::Fn(fndecl))
    }).collect();

    if let Some(framework_name) = framework_name {
        ast.items.push(parse_quote!{
            #[link(name=#framework_name, kind="framework")]
            extern "C" {
                #(#funcs)*
            }
        });
    } else if !funcs.is_empty() {
        ast.items.push(parse_quote!{
            extern "C" {
                #(#funcs)*
            }
        });
    }

    let mut f = File::create(out_path).unwrap();
    f.write_fmt(format_args!("{}", ast.into_token_stream())).unwrap();
    f.flush().unwrap();
    std::process::Command::new("rustfmt").arg(out_path).status().unwrap();
}
