// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::ptr;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;
use std::mem;
use std::ffi::{CStr, CString};
use clang::*;

#[derive(Debug, PartialEq)]
pub enum CursorKind {
    UnexposedDecl,
    StructDecl,
    UnionDecl,
    ClassDecl,
    EnumDecl,
    FieldDecl,
    EnumConstantDecl,
    FunctionDecl,
    VarDecl,
    ParmDecl,
    ObjCInterfaceDecl,
    ObjCCategoryDecl,
    ObjCProtocolDecl,
    ObjCPropertyDecl,
    ObjCIvarDecl,
    ObjCInstanceMethodDecl,
    ObjCClassMethodDecl,
    ObjCImplementationDecl,
    ObjCCategoryImplDecl,
    TypedefDecl,
    CXXMethod,
    Namespace,
    LinkageSpec,
    Constructor,
    Destructor,
    ConversionFunction,
    TemplateTypeParameter,
    NonTypeTemplateParameter,
    TemplateTemplateParameter,
    FunctionTemplate,
    ClassTemplate,
    ClassTemplatePartialSpecialization,
    NamespaceAlias,
    UsingDirective,
    UsingDeclaration,
    TypeAliasDecl,
    ObjCSynthesizeDecl,
    ObjCDynamicDecl,
    CXXAccessSpecifier,
    ObjCSuperClassRef,
    ObjCProtocolRef,
    ObjCClassRef,
    TypeRef,
    CXXBaseSpecifier,
    TemplateRef,
    NamespaceRef,
    MemberRef,
    LabelRef,
    OverloadedDeclRef,
    VariableRef,
    InvalidFile,
    NoDeclFound,
    NotImplemented,
    InvalidCode,
    UnexposedExpr,
    DeclRefExpr,
    MemberRefExpr,
    CallExpr,
    ObjCMessageExpr,
    BlockExpr,
    IntegerLiteral,
    FloatingLiteral,
    ImaginaryLiteral,
    StringLiteral,
    CharacterLiteral,
    ParenExpr,
    UnaryOperator,
    ArraySubscriptExpr,
    BinaryOperator,
    CompoundAssignOperator,
    ConditionalOperator,
    CStyleCastExpr,
    CompoundLiteralExpr,
    InitListExpr,
    AddrLabelExpr,
    StmtExpr,
    GenericSelectionExpr,
    GNUNullExpr,
    CXXStaticCastExpr,
    CXXDynamicCastExpr,
    CXXReinterpretCastExpr,
    CXXConstCastExpr,
    CXXFunctionalCastExpr,
    CXXTypeidExpr,
    CXXBoolLiteralExpr,
    CXXNullPtrLiteralExpr,
    CXXThisExpr,
    CXXThrowExpr,
    CXXNewExpr,
    CXXDeleteExpr,
    UnaryExpr,
    ObjCStringLiteral,
    ObjCEncodeExpr,
    ObjCSelectorExpr,
    ObjCProtocolExpr,
    ObjCBridgedCastExpr,
    PackExpansionExpr,
    SizeOfPackExpr,
    LambdaExpr,
    ObjCBoolLiteralExpr,
    ObjCSelfExpr,
    /// Only produced by `libclang` 3.8 and later.
    OMPArraySectionExpr,
    /// Only produced by `libclang` 3.9 and later.
    ObjCAvailabilityCheckExpr,
    UnexposedStmt,
    LabelStmt,
    CompoundStmt,
    CaseStmt,
    DefaultStmt,
    IfStmt,
    SwitchStmt,
    WhileStmt,
    DoStmt,
    ForStmt,
    GotoStmt,
    IndirectGotoStmt,
    ContinueStmt,
    BreakStmt,
    ReturnStmt,
    /// Duplicate of `CXCursor_GccAsmStmt`.
    AsmStmt,
    ObjCAtTryStmt,
    ObjCAtCatchStmt,
    ObjCAtFinallyStmt,
    ObjCAtThrowStmt,
    ObjCAtSynchronizedStmt,
    ObjCAutoreleasePoolStmt,
    ObjCForCollectionStmt,
    CXXCatchStmt,
    CXXTryStmt,
    CXXForRangeStmt,
    SEHTryStmt,
    SEHExceptStmt,
    SEHFinallyStmt,
    MSAsmStmt,
    NullStmt,
    DeclStmt,
    OMPParallelDirective,
    OMPSimdDirective,
    OMPForDirective,
    OMPSectionsDirective,
    OMPSectionDirective,
    OMPSingleDirective,
    OMPParallelForDirective,
    OMPParallelSectionsDirective,
    OMPTaskDirective,
    OMPMasterDirective,
    OMPCriticalDirective,
    OMPTaskyieldDirective,
    OMPBarrierDirective,
    OMPTaskwaitDirective,
    OMPFlushDirective,
    SEHLeaveStmt,
    /// Only produced by `libclang` 3.6 and later.
    OMPOrderedDirective,
    /// Only produced by `libclang` 3.6 and later.
    OMPAtomicDirective,
    /// Only produced by `libclang` 3.6 and later.
    OMPForSimdDirective,
    /// Only produced by `libclang` 3.6 and later.
    OMPParallelForSimdDirective,
    /// Only produced by `libclang` 3.6 and later.
    OMPTargetDirective,
    /// Only produced by `libclang` 3.6 and later.
    OMPTeamsDirective,
    /// Only produced by `libclang` 3.7 and later.
    OMPTaskgroupDirective,
    /// Only produced by `libclang` 3.7 and later.
    OMPCancellationPointDirective,
    /// Only produced by `libclang` 3.7 and later.
    OMPCancelDirective,
    /// Only produced by `libclang` 3.8 and later.
    OMPTargetDataDirective,
    /// Only produced by `libclang` 3.8 and later.
    OMPTaskLoopDirective,
    /// Only produced by `libclang` 3.8 and later.
    OMPTaskLoopSimdDirective,
    /// Only produced by `libclang` 3.8 and later.
    OMPDistributeDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetEnterDataDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetExitDataDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetParallelDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetParallelForDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetUpdateDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPDistributeParallelForDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPDistributeParallelForSimdDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPDistributeSimdDirective,
    /// Only produced by `libclang` 3.9 and later.
    OMPTargetParallelForSimdDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTargetSimdDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTeamsDistributeDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTeamsDistributeSimdDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTeamsDistributeParallelForSimdDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTeamsDistributeParallelForDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTargetTeamsDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTargetTeamsDistributeDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTargetTeamsDistributeParallelForDirective,
    /// Only produced by `libclang` 4.0 and later.
    OMPTargetTeamsDistributeParallelForSimdDirective,
    /// Only producer by `libclang` 4.0 and later.
    OMPTargetTeamsDistributeSimdDirective,
    TranslationUnit,
    UnexposedAttr,
    IBActionAttr,
    IBOutletAttr,
    IBOutletCollectionAttr,
    CXXFinalAttr,
    CXXOverrideAttr,
    AnnotateAttr,
    AsmLabelAttr,
    PackedAttr,
    PureAttr,
    ConstAttr,
    NoDuplicateAttr,
    CUDAConstantAttr,
    CUDADeviceAttr,
    CUDAGlobalAttr,
    CUDAHostAttr,
    /// Only produced by `libclang` 3.6 and later.
    CUDASharedAttr,
    /// Only produced by `libclang` 3.8 and later.
    VisibilityAttr,
    /// Only produced by `libclang` 3.8 and later.
    DLLExport,
    /// Only produced by `libclang` 3.8 and later.
    DLLImport,
    /// Only produced by `libclang` 7.0 and later.
    NSReturnsRetained,
    /// Only produced by `libclang` 7.0 and later.
    NSReturnsNotRetained,
    /// Only produced by `libclang` 7.0 and later.
    NSReturnsAutoreleased,
    /// Only produced by `libclang` 7.0 and later.
    NSConsumesSelf,
    /// Only produced by `libclang` 7.0 and later.
    NSConsumed,
    /// Only produced by `libclang` 7.0 and later.
    ObjCException,
    /// Only produced by `libclang` 7.0 and later.
    ObjCNSObject,
    /// Only produced by `libclang` 7.0 and later.
    ObjCIndependentClass,
    /// Only produced by `libclang` 7.0 and later.
    ObjCPreciseLifetime,
    /// Only produced by `libclang` 7.0 and later.
    ObjCReturnsInnerPointer,
    /// Only produced by `libclang` 7.0 and later.
    ObjCRequiresSuper,
    /// Only produced by `libclang` 7.0 and later.
    ObjCRootClass,
    /// Only produced by `libclang` 7.0 and later.
    ObjCSubclassingRestricted,
    /// Only produced by `libclang` 7.0 and later.
    ObjCExplicitProtocolImpl,
    /// Only produced by `libclang` 7.0 and later.
    ObjCDesignatedInitializer,
    /// Only produced by `libclang` 7.0 and later.
    ObjCRuntimeVisible,
    /// Only produced by `libclang` 7.0 and later.
    ObjCBoxable,
    /// Only produced by `libclang` 7.0 and later.
    FlagEnum,
    PreprocessingDirective,
    MacroDefinition,
    /// Duplicate of `CXCursor_MacroInstantiation`.
    MacroExpansion,
    InclusionDirective,
    ModuleImportDecl,
    /// Only produced by `libclang` 3.8 and later.
    TypeAliasTemplateDecl,
    /// Only produced by `libclang` 3.9 and later.
    StaticAssert,
    /// Only produced by `libclang` 4.0 and later.
    FriendDecl,
    /// Only produced by `libclang` 3.7 and later.
    OverloadCandidate,
}

impl CursorKind {
    #[allow(non_upper_case_globals)]
    pub fn from_raw(e: CXCursorKind) -> CursorKind {
        match e {
            CXCursor_UnexposedDecl => CursorKind::UnexposedDecl,
            CXCursor_StructDecl => CursorKind::StructDecl,
            CXCursor_UnionDecl => CursorKind::UnionDecl,
            CXCursor_ClassDecl => CursorKind::ClassDecl,
            CXCursor_EnumDecl => CursorKind::EnumDecl,
            CXCursor_FieldDecl => CursorKind::FieldDecl,
            CXCursor_EnumConstantDecl => CursorKind::EnumConstantDecl,
            CXCursor_FunctionDecl => CursorKind::FunctionDecl,
            CXCursor_VarDecl => CursorKind::VarDecl,
            CXCursor_ParmDecl => CursorKind::ParmDecl,
            CXCursor_ObjCInterfaceDecl => CursorKind::ObjCInterfaceDecl,
            CXCursor_ObjCCategoryDecl => CursorKind::ObjCCategoryDecl,
            CXCursor_ObjCProtocolDecl => CursorKind::ObjCProtocolDecl,
            CXCursor_ObjCPropertyDecl => CursorKind::ObjCPropertyDecl,
            CXCursor_ObjCIvarDecl => CursorKind::ObjCIvarDecl,
            CXCursor_ObjCInstanceMethodDecl =>
                CursorKind::ObjCInstanceMethodDecl,
            CXCursor_ObjCClassMethodDecl => CursorKind::ObjCClassMethodDecl,
            CXCursor_ObjCImplementationDecl =>
                CursorKind::ObjCImplementationDecl,
            CXCursor_ObjCCategoryImplDecl => CursorKind::ObjCCategoryImplDecl,
            CXCursor_TypedefDecl => CursorKind::TypedefDecl,
            CXCursor_CXXMethod => CursorKind::CXXMethod,
            CXCursor_Namespace => CursorKind::Namespace,
            CXCursor_LinkageSpec => CursorKind::LinkageSpec,
            CXCursor_Constructor => CursorKind::Constructor,
            CXCursor_Destructor => CursorKind::Destructor,
            CXCursor_ConversionFunction => CursorKind::ConversionFunction,
            CXCursor_TemplateTypeParameter => CursorKind::TemplateTypeParameter,
            CXCursor_NonTypeTemplateParameter =>
                CursorKind::NonTypeTemplateParameter,
            CXCursor_TemplateTemplateParameter =>
                CursorKind::TemplateTemplateParameter,
            CXCursor_FunctionTemplate => CursorKind::FunctionTemplate,
            CXCursor_ClassTemplate => CursorKind::ClassTemplate,
            CXCursor_ClassTemplatePartialSpecialization =>
                CursorKind::ClassTemplatePartialSpecialization,
            CXCursor_NamespaceAlias => CursorKind::NamespaceAlias,
            CXCursor_UsingDirective => CursorKind::UsingDirective,
            CXCursor_UsingDeclaration => CursorKind::UsingDeclaration,
            CXCursor_TypeAliasDecl => CursorKind::TypeAliasDecl,
            CXCursor_ObjCSynthesizeDecl => CursorKind::ObjCSynthesizeDecl,
            CXCursor_ObjCDynamicDecl => CursorKind::ObjCDynamicDecl,
            CXCursor_CXXAccessSpecifier => CursorKind::CXXAccessSpecifier,
            CXCursor_ObjCSuperClassRef => CursorKind::ObjCSuperClassRef,
            CXCursor_ObjCProtocolRef => CursorKind::ObjCProtocolRef,
            CXCursor_ObjCClassRef => CursorKind::ObjCClassRef,
            CXCursor_TypeRef => CursorKind::TypeRef,
            CXCursor_CXXBaseSpecifier => CursorKind::CXXBaseSpecifier,
            CXCursor_TemplateRef => CursorKind::TemplateRef,
            CXCursor_NamespaceRef => CursorKind::NamespaceRef,
            CXCursor_MemberRef => CursorKind::MemberRef,
            CXCursor_LabelRef => CursorKind::LabelRef,
            CXCursor_OverloadedDeclRef => CursorKind::OverloadedDeclRef,
            CXCursor_VariableRef => CursorKind::VariableRef,
            CXCursor_InvalidFile => CursorKind::InvalidFile,
            CXCursor_NoDeclFound => CursorKind::NoDeclFound,
            CXCursor_NotImplemented => CursorKind::NotImplemented,
            CXCursor_InvalidCode => CursorKind::InvalidCode,
            CXCursor_UnexposedExpr => CursorKind::UnexposedExpr,
            CXCursor_DeclRefExpr => CursorKind::DeclRefExpr,
            CXCursor_MemberRefExpr => CursorKind::MemberRefExpr,
            CXCursor_CallExpr => CursorKind::CallExpr,
            CXCursor_ObjCMessageExpr => CursorKind::ObjCMessageExpr,
            CXCursor_BlockExpr => CursorKind::BlockExpr,
            CXCursor_IntegerLiteral => CursorKind::IntegerLiteral,
            CXCursor_FloatingLiteral => CursorKind::FloatingLiteral,
            CXCursor_ImaginaryLiteral => CursorKind::ImaginaryLiteral,
            CXCursor_StringLiteral => CursorKind::StringLiteral,
            CXCursor_CharacterLiteral => CursorKind::CharacterLiteral,
            CXCursor_ParenExpr => CursorKind::ParenExpr,
            CXCursor_UnaryOperator => CursorKind::UnaryOperator,
            CXCursor_ArraySubscriptExpr => CursorKind::ArraySubscriptExpr,
            CXCursor_BinaryOperator => CursorKind::BinaryOperator,
            CXCursor_CompoundAssignOperator =>
                CursorKind::CompoundAssignOperator,
            CXCursor_ConditionalOperator => CursorKind::ConditionalOperator,
            CXCursor_CStyleCastExpr => CursorKind::CStyleCastExpr,
            CXCursor_CompoundLiteralExpr => CursorKind::CompoundLiteralExpr,
            CXCursor_InitListExpr => CursorKind::InitListExpr,
            CXCursor_AddrLabelExpr => CursorKind::AddrLabelExpr,
            CXCursor_StmtExpr => CursorKind::StmtExpr,
            CXCursor_GenericSelectionExpr => CursorKind::GenericSelectionExpr,
            CXCursor_GNUNullExpr => CursorKind::GNUNullExpr,
            CXCursor_CXXStaticCastExpr => CursorKind::CXXStaticCastExpr,
            CXCursor_CXXDynamicCastExpr => CursorKind::CXXDynamicCastExpr,
            CXCursor_CXXReinterpretCastExpr =>
                CursorKind::CXXReinterpretCastExpr,
            CXCursor_CXXConstCastExpr => CursorKind::CXXConstCastExpr,
            CXCursor_CXXFunctionalCastExpr => CursorKind::CXXFunctionalCastExpr,
            CXCursor_CXXTypeidExpr => CursorKind::CXXTypeidExpr,
            CXCursor_CXXBoolLiteralExpr => CursorKind::CXXBoolLiteralExpr,
            CXCursor_CXXNullPtrLiteralExpr => CursorKind::CXXNullPtrLiteralExpr,
            CXCursor_CXXThisExpr => CursorKind::CXXThisExpr,
            CXCursor_CXXThrowExpr => CursorKind::CXXThrowExpr,
            CXCursor_CXXNewExpr => CursorKind::CXXNewExpr,
            CXCursor_CXXDeleteExpr => CursorKind::CXXDeleteExpr,
            CXCursor_UnaryExpr => CursorKind::UnaryExpr,
            CXCursor_ObjCStringLiteral => CursorKind::ObjCStringLiteral,
            CXCursor_ObjCEncodeExpr => CursorKind::ObjCEncodeExpr,
            CXCursor_ObjCSelectorExpr => CursorKind::ObjCSelectorExpr,
            CXCursor_ObjCProtocolExpr => CursorKind::ObjCProtocolExpr,
            CXCursor_ObjCBridgedCastExpr => CursorKind::ObjCBridgedCastExpr,
            CXCursor_PackExpansionExpr => CursorKind::PackExpansionExpr,
            CXCursor_SizeOfPackExpr => CursorKind::SizeOfPackExpr,
            CXCursor_LambdaExpr => CursorKind::LambdaExpr,
            CXCursor_ObjCBoolLiteralExpr => CursorKind::ObjCBoolLiteralExpr,
            CXCursor_ObjCSelfExpr => CursorKind::ObjCSelfExpr,
            CXCursor_OMPArraySectionExpr => CursorKind::OMPArraySectionExpr,
            CXCursor_ObjCAvailabilityCheckExpr =>
                CursorKind::ObjCAvailabilityCheckExpr,
            CXCursor_UnexposedStmt => CursorKind::UnexposedStmt,
            CXCursor_LabelStmt => CursorKind::LabelStmt,
            CXCursor_CompoundStmt => CursorKind::CompoundStmt,
            CXCursor_CaseStmt => CursorKind::CaseStmt,
            CXCursor_DefaultStmt => CursorKind::DefaultStmt,
            CXCursor_IfStmt => CursorKind::IfStmt,
            CXCursor_SwitchStmt => CursorKind::SwitchStmt,
            CXCursor_WhileStmt => CursorKind::WhileStmt,
            CXCursor_DoStmt => CursorKind::DoStmt,
            CXCursor_ForStmt => CursorKind::ForStmt,
            CXCursor_GotoStmt => CursorKind::GotoStmt,
            CXCursor_IndirectGotoStmt => CursorKind::IndirectGotoStmt,
            CXCursor_ContinueStmt => CursorKind::ContinueStmt,
            CXCursor_BreakStmt => CursorKind::BreakStmt,
            CXCursor_ReturnStmt => CursorKind::ReturnStmt,
            CXCursor_AsmStmt => CursorKind::AsmStmt,
            CXCursor_ObjCAtTryStmt => CursorKind::ObjCAtTryStmt,
            CXCursor_ObjCAtCatchStmt => CursorKind::ObjCAtCatchStmt,
            CXCursor_ObjCAtFinallyStmt => CursorKind::ObjCAtFinallyStmt,
            CXCursor_ObjCAtThrowStmt => CursorKind::ObjCAtThrowStmt,
            CXCursor_ObjCAtSynchronizedStmt =>
                CursorKind::ObjCAtSynchronizedStmt,
            CXCursor_ObjCAutoreleasePoolStmt =>
                CursorKind::ObjCAutoreleasePoolStmt,
            CXCursor_ObjCForCollectionStmt => CursorKind::ObjCForCollectionStmt,
            CXCursor_CXXCatchStmt => CursorKind::CXXCatchStmt,
            CXCursor_CXXTryStmt => CursorKind::CXXTryStmt,
            CXCursor_CXXForRangeStmt => CursorKind::CXXForRangeStmt,
            CXCursor_SEHTryStmt => CursorKind::SEHTryStmt,
            CXCursor_SEHExceptStmt => CursorKind::SEHExceptStmt,
            CXCursor_SEHFinallyStmt => CursorKind::SEHFinallyStmt,
            CXCursor_MSAsmStmt => CursorKind::MSAsmStmt,
            CXCursor_NullStmt => CursorKind::NullStmt,
            CXCursor_DeclStmt => CursorKind::DeclStmt,
            CXCursor_OMPParallelDirective => CursorKind::OMPParallelDirective,
            CXCursor_OMPSimdDirective => CursorKind::OMPSimdDirective,
            CXCursor_OMPForDirective => CursorKind::OMPForDirective,
            CXCursor_OMPSectionsDirective => CursorKind::OMPSectionsDirective,
            CXCursor_OMPSectionDirective => CursorKind::OMPSectionDirective,
            CXCursor_OMPSingleDirective => CursorKind::OMPSingleDirective,
            CXCursor_OMPParallelForDirective =>
                CursorKind::OMPParallelForDirective,
            CXCursor_OMPParallelSectionsDirective =>
                CursorKind::OMPParallelSectionsDirective,
            CXCursor_OMPTaskDirective => CursorKind::OMPTaskDirective,
            CXCursor_OMPMasterDirective => CursorKind::OMPMasterDirective,
            CXCursor_OMPCriticalDirective => CursorKind::OMPCriticalDirective,
            CXCursor_OMPTaskyieldDirective => CursorKind::OMPTaskyieldDirective,
            CXCursor_OMPBarrierDirective => CursorKind::OMPBarrierDirective,
            CXCursor_OMPTaskwaitDirective => CursorKind::OMPTaskwaitDirective,
            CXCursor_OMPFlushDirective => CursorKind::OMPFlushDirective,
            CXCursor_SEHLeaveStmt => CursorKind::SEHLeaveStmt,
            CXCursor_OMPOrderedDirective => CursorKind::OMPOrderedDirective,
            CXCursor_OMPAtomicDirective => CursorKind::OMPAtomicDirective,
            CXCursor_OMPForSimdDirective => CursorKind::OMPForSimdDirective,
            CXCursor_OMPParallelForSimdDirective =>
                CursorKind::OMPParallelForSimdDirective,
            CXCursor_OMPTargetDirective => CursorKind::OMPTargetDirective,
            CXCursor_OMPTeamsDirective => CursorKind::OMPTeamsDirective,
            CXCursor_OMPTaskgroupDirective => CursorKind::OMPTaskgroupDirective,
            CXCursor_OMPCancellationPointDirective =>
                CursorKind::OMPCancellationPointDirective,
            CXCursor_OMPCancelDirective => CursorKind::OMPCancelDirective,
            CXCursor_OMPTargetDataDirective =>
                CursorKind::OMPTargetDataDirective,
            CXCursor_OMPTaskLoopDirective => CursorKind::OMPTaskLoopDirective,
            CXCursor_OMPTaskLoopSimdDirective =>
                CursorKind::OMPTaskLoopSimdDirective,
            CXCursor_OMPDistributeDirective =>
                CursorKind::OMPDistributeDirective,
            CXCursor_OMPTargetEnterDataDirective =>
                CursorKind::OMPTargetEnterDataDirective,
            CXCursor_OMPTargetExitDataDirective =>
                CursorKind::OMPTargetExitDataDirective,
            CXCursor_OMPTargetParallelDirective =>
                CursorKind::OMPTargetParallelDirective,
            CXCursor_OMPTargetParallelForDirective =>
                CursorKind::OMPTargetParallelForDirective,
            CXCursor_OMPTargetUpdateDirective =>
                CursorKind::OMPTargetUpdateDirective,
            CXCursor_OMPDistributeParallelForDirective =>
                CursorKind::OMPDistributeParallelForDirective,
            CXCursor_OMPDistributeParallelForSimdDirective =>
                CursorKind::OMPDistributeParallelForSimdDirective,
            CXCursor_OMPDistributeSimdDirective =>
                CursorKind::OMPDistributeSimdDirective,
            CXCursor_OMPTargetParallelForSimdDirective =>
                CursorKind::OMPTargetParallelForSimdDirective,
            CXCursor_OMPTargetSimdDirective =>
                CursorKind::OMPTargetSimdDirective,
            CXCursor_OMPTeamsDistributeDirective =>
                CursorKind::OMPTeamsDistributeDirective,
            CXCursor_OMPTeamsDistributeSimdDirective =>
                CursorKind::OMPTeamsDistributeSimdDirective,
            CXCursor_OMPTeamsDistributeParallelForSimdDirective =>
                CursorKind::OMPTeamsDistributeParallelForSimdDirective,
            CXCursor_OMPTeamsDistributeParallelForDirective =>
                CursorKind::OMPTeamsDistributeParallelForDirective,
            CXCursor_OMPTargetTeamsDirective =>
                CursorKind::OMPTargetTeamsDirective,
            CXCursor_OMPTargetTeamsDistributeDirective =>
                CursorKind::OMPTargetTeamsDistributeDirective,
            CXCursor_OMPTargetTeamsDistributeParallelForDirective =>
                CursorKind::OMPTargetTeamsDistributeParallelForDirective,
            CXCursor_OMPTargetTeamsDistributeParallelForSimdDirective =>
                CursorKind::OMPTargetTeamsDistributeParallelForSimdDirective,
            CXCursor_OMPTargetTeamsDistributeSimdDirective =>
                CursorKind::OMPTargetTeamsDistributeSimdDirective,
            CXCursor_TranslationUnit => CursorKind::TranslationUnit,
            CXCursor_UnexposedAttr => CursorKind::UnexposedAttr,
            CXCursor_IBActionAttr => CursorKind::IBActionAttr,
            CXCursor_IBOutletAttr => CursorKind::IBOutletAttr,
            CXCursor_IBOutletCollectionAttr =>
                CursorKind::IBOutletCollectionAttr,
            CXCursor_CXXFinalAttr => CursorKind::CXXFinalAttr,
            CXCursor_CXXOverrideAttr => CursorKind::CXXOverrideAttr,
            CXCursor_AnnotateAttr => CursorKind::AnnotateAttr,
            CXCursor_AsmLabelAttr => CursorKind::AsmLabelAttr,
            CXCursor_PackedAttr => CursorKind::PackedAttr,
            CXCursor_PureAttr => CursorKind::PureAttr,
            CXCursor_ConstAttr => CursorKind::ConstAttr,
            CXCursor_NoDuplicateAttr => CursorKind::NoDuplicateAttr,
            CXCursor_CUDAConstantAttr => CursorKind::CUDAConstantAttr,
            CXCursor_CUDADeviceAttr => CursorKind::CUDADeviceAttr,
            CXCursor_CUDAGlobalAttr => CursorKind::CUDAGlobalAttr,
            CXCursor_CUDAHostAttr => CursorKind::CUDAHostAttr,
            CXCursor_CUDASharedAttr => CursorKind::CUDASharedAttr,
            CXCursor_VisibilityAttr => CursorKind::VisibilityAttr,
            CXCursor_NSReturnsRetained => CursorKind::NSReturnsRetained,
            CXCursor_NSReturnsNotRetained => CursorKind::NSReturnsNotRetained,
            CXCursor_NSReturnsAutoreleased => CursorKind::NSReturnsAutoreleased,
            CXCursor_NSConsumesSelf => CursorKind::NSConsumesSelf,
            CXCursor_NSConsumed => CursorKind::NSConsumed,
            CXCursor_ObjCException => CursorKind::ObjCException,
            CXCursor_ObjCNSObject => CursorKind::ObjCNSObject,
            CXCursor_ObjCIndependentClass => CursorKind::ObjCIndependentClass,
            CXCursor_ObjCPreciseLifetime => CursorKind::ObjCPreciseLifetime,
            CXCursor_ObjCReturnsInnerPointer => CursorKind::ObjCReturnsInnerPointer,
            CXCursor_ObjCRequiresSuper => CursorKind::ObjCRequiresSuper,
            CXCursor_ObjCRootClass => CursorKind::ObjCRootClass,
            CXCursor_ObjCSubclassingRestricted => CursorKind::ObjCSubclassingRestricted,
            CXCursor_ObjCExplicitProtocolImpl => CursorKind::ObjCExplicitProtocolImpl,
            CXCursor_ObjCDesignatedInitializer => CursorKind::ObjCDesignatedInitializer,
            CXCursor_ObjCRuntimeVisible => CursorKind::ObjCRuntimeVisible,
            CXCursor_ObjCBoxable => CursorKind::ObjCBoxable,
            CXCursor_FlagEnum => CursorKind::FlagEnum,
            CXCursor_DLLExport => CursorKind::DLLExport,
            CXCursor_DLLImport => CursorKind::DLLImport,
            CXCursor_PreprocessingDirective =>
                CursorKind::PreprocessingDirective,
            CXCursor_MacroDefinition => CursorKind::MacroDefinition,
            CXCursor_MacroExpansion => CursorKind::MacroExpansion,
            CXCursor_InclusionDirective => CursorKind::InclusionDirective,
            CXCursor_ModuleImportDecl => CursorKind::ModuleImportDecl,
            CXCursor_TypeAliasTemplateDecl => CursorKind::TypeAliasTemplateDecl,
            CXCursor_StaticAssert => CursorKind::StaticAssert,
            CXCursor_FriendDecl => CursorKind::FriendDecl,
            CXCursor_OverloadCandidate => CursorKind::OverloadCandidate,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TypeKind {
    Invalid,
    Unexposed,
    Void,
    Bool,
    Char_U,
    UChar,
    Char16,
    Char32,
    UShort,
    UInt,
    ULong,
    ULongLong,
    UInt128,
    Char_S,
    SChar,
    WChar,
    Short,
    Int,
    Long,
    LongLong,
    Int128,
    Float,
    Double,
    LongDouble,
    NullPtr,
    Overload,
    Dependent,
    ObjCId,
    ObjCClass,
    ObjCSel,
    /// Only produced by `libclang` 3.9 and later.
    Float128,
    /// Only produced by `libclang` 5.0 and later.
    Half,
    /// Only produced by `libclang` 6.0 and later.
    Float16,
    Complex,
    Pointer,
    BlockPointer,
    LValueReference,
    RValueReference,
    Record,
    Enum,
    Typedef,
    ObjCInterface,
    ObjCObjectPointer,
    FunctionNoProto,
    FunctionProto,
    ConstantArray,
    Vector,
    IncompleteArray,
    VariableArray,
    DependentSizedArray,
    MemberPointer,
    /// Only produced by `libclang` 3.8 and later.
    Auto,
    /// Only produced by `libclang` 3.9 and later.
    Elaborated,
    /// Only produced by `libclang` 5.0 and later.
    Pipe,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dArrayRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dBufferRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dDepthRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayDepthRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAARO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAARO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAADepthRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAADepthRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage3dRO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dArrayWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dBufferWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dDepthWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayDepthWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAAWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAAWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAADepthWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAADepthWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage3dWO,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dArrayRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage1dBufferRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dDepthRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayDepthRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAARW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAARW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dMSAADepthRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage2dArrayMSAADepthRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLImage3dRW,
    /// Only produced by `libclang` 5.0 and later.
    OCLSampler,
    /// Only produced by `libclang` 5.0 and later.
    OCLEvent,
    /// Only produced by `libclang` 5.0 and later.
    OCLQueue,
    /// Only produced by `libclang` 5.0 and later.
    OCLReserveID,
    /// Only produced by `libclang` 7.0 and later.
    ObjCObject,
    /// Only produced by `libclang` 7.0 and later.
    ObjCTypeParam,
    /// Only produced by `libclang` 7.0 and later.
    Attributed,
}

#[derive(Debug, PartialEq)]
pub enum Nullability {
    NonNull,
    Nullable,
    Unspecified,
}

#[derive(Debug, PartialEq)]
pub enum ChildVisit {
    Break = CXChildVisit_Break as isize,
    Continue = CXChildVisit_Continue as isize,
    Recurse = CXChildVisit_Recurse as isize,
}

fn into_str(s: CXString) -> String {
    if s.data.is_null() {
        return "".to_owned();
    }
    let c_str = unsafe { CStr::from_ptr(clang_getCString(s) as *const _) };
    let out = c_str.to_string_lossy().into_owned();
    unsafe { clang_disposeString(s) }
    out
}

pub struct SourceLocation {
    loc: CXSourceLocation,
}

impl SourceLocation {
    pub fn filename(&self) -> PathBuf {
        let mut file = ptr::null_mut();
        let mut line = 0u32;
        let mut column = 0u32;
        let mut offset = 0u32;
        let name;
        unsafe {
            clang_getFileLocation(self.loc, &mut file as *mut _, &mut line as *mut _, &mut column as *mut _, &mut offset as *mut _);
            name = clang_getFileName(file);
        }
        PathBuf::from(into_str(name))
    }
}

impl PartialEq for SourceLocation {
    fn eq(&self, other: &SourceLocation) -> bool {
        unsafe { clang_equalLocations(self.loc, other.loc) != 0 }
    }
}

pub struct PropertyAttributes {
    attr: i32,
}

impl PropertyAttributes {
    pub fn readonly(&self) -> bool {
        self.attr & CXObjCPropertyAttr_readonly != 0
    }

    pub fn getter(&self) -> bool {
        self.attr & CXObjCPropertyAttr_getter != 0
    }

    pub fn assign(&self) -> bool {
        self.attr & CXObjCPropertyAttr_assign != 0
    }

    pub fn readwrite(&self) -> bool {
        self.attr & CXObjCPropertyAttr_readwrite != 0
    }

    pub fn retain(&self) -> bool {
        self.attr & CXObjCPropertyAttr_retain != 0
    }

    pub fn copy(&self) -> bool {
        self.attr & CXObjCPropertyAttr_copy != 0
    }

    pub fn nonatomic(&self) -> bool {
        self.attr & CXObjCPropertyAttr_nonatomic != 0
    }

    pub fn setter(&self) -> bool {
        self.attr & CXObjCPropertyAttr_setter != 0
    }

    pub fn atomic(&self) -> bool {
        self.attr & CXObjCPropertyAttr_atomic != 0
    }

    pub fn weak(&self) -> bool {
        self.attr & CXObjCPropertyAttr_weak != 0
    }

    pub fn strong(&self) -> bool {
        self.attr & CXObjCPropertyAttr_strong != 0
    }

    pub fn unsafe_unretained(&self) -> bool {
        self.attr & CXObjCPropertyAttr_unsafe_unretained != 0
    }

    pub fn class(&self) -> bool {
        self.attr & CXObjCPropertyAttr_class != 0
    }
}

pub struct FunctionArgIterator<'a> {
    t: &'a Ty,
    i: u32,
}

impl<'a> Iterator for FunctionArgIterator<'a> {
    type Item = Ty;

    fn next(&mut self) -> Option<Ty> {
        let idx = self.i;
        self.i += 1;
        self.t.arg(idx)
    }
}

impl<'a> ExactSizeIterator for FunctionArgIterator<'a> {
    fn len(&self) -> usize {
        self.t.num_args() as usize
    }
}

pub struct ObjCTypeArgIterator<'a> {
    t: &'a Ty,
    i: u32,
}

impl<'a> Iterator for ObjCTypeArgIterator<'a> {
    type Item = Ty;

    fn next(&mut self) -> Option<Ty> {
        let idx = self.i;
        self.i += 1;
        self.t.type_arg(idx)
    }
}

impl<'a> ExactSizeIterator for ObjCTypeArgIterator<'a> {
    fn len(&self) -> usize {
        self.t.num_type_args() as usize
    }
}

pub struct ObjCProtocolIterator<'a> {
    t: &'a Ty,
    i: u32,
}

impl<'a> Iterator for ObjCProtocolIterator<'a> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Cursor> {
        let idx = self.i;
        self.i += 1;
        if self.t.num_protocols() > idx {
            self.t.protocol_ref(idx)
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for ObjCProtocolIterator<'a> {
    fn len(&self) -> usize {
        self.t.num_protocols() as usize
    }
}

pub struct CursorArgIterator<'a> {
    c: &'a Cursor,
    i: u32,
}

impl<'a> Iterator for CursorArgIterator<'a> {
    type Item = Cursor;

    fn next(&mut self) -> Option<Cursor> {
        let idx = self.i;
        self.i += 1;
        if self.c.num_args() > idx {
            Some(self.c.arg(idx))
        } else {
            None
        }
    }
}

impl<'a> ExactSizeIterator for CursorArgIterator<'a> {
    fn len(&self) -> usize {
        self.c.num_args() as usize
    }                
}

pub struct Ty {
    t: CXType,
}

impl Ty {
    #[allow(non_upper_case_globals)]
    pub fn kind(&self) -> TypeKind {
        match self.t.kind {
            CXType_Invalid => TypeKind::Invalid,
            CXType_Unexposed => TypeKind::Unexposed,
            CXType_Void => TypeKind::Void,
            CXType_Bool => TypeKind::Bool,
            CXType_Char_U => TypeKind::Char_U,
            CXType_UChar => TypeKind::UChar,
            CXType_Char16 => TypeKind::Char16,
            CXType_Char32 => TypeKind::Char32,
            CXType_UShort => TypeKind::UShort,
            CXType_UInt => TypeKind::UInt,
            CXType_ULong => TypeKind::ULong,
            CXType_ULongLong => TypeKind::ULongLong,
            CXType_UInt128 => TypeKind::UInt128,
            CXType_Char_S => TypeKind::Char_S,
            CXType_SChar => TypeKind::SChar,
            CXType_WChar => TypeKind::WChar,
            CXType_Short => TypeKind::Short,
            CXType_Int => TypeKind::Int,
            CXType_Long => TypeKind::Long,
            CXType_LongLong => TypeKind::LongLong,
            CXType_Int128 => TypeKind::Int128,
            CXType_Float => TypeKind::Float,
            CXType_Double => TypeKind::Double,
            CXType_LongDouble => TypeKind::LongDouble,
            CXType_NullPtr => TypeKind::NullPtr,
            CXType_Overload => TypeKind::Overload,
            CXType_Dependent => TypeKind::Dependent,
            CXType_ObjCId => TypeKind::ObjCId,
            CXType_ObjCClass => TypeKind::ObjCClass,
            CXType_ObjCSel => TypeKind::ObjCSel,
            CXType_Float128 => TypeKind::Float128,
            CXType_Half => TypeKind::Half,
            CXType_Float16 => TypeKind::Float16,
            CXType_Complex => TypeKind::Complex,
            CXType_Pointer => TypeKind::Pointer,
            CXType_BlockPointer => TypeKind::BlockPointer,
            CXType_LValueReference => TypeKind::LValueReference,
            CXType_RValueReference => TypeKind::RValueReference,
            CXType_Record => TypeKind::Record,
            CXType_Enum => TypeKind::Enum,
            CXType_Typedef => TypeKind::Typedef,
            CXType_ObjCInterface => TypeKind::ObjCInterface,
            CXType_ObjCObjectPointer => TypeKind::ObjCObjectPointer,
            CXType_FunctionNoProto => TypeKind::FunctionNoProto,
            CXType_FunctionProto => TypeKind::FunctionProto,
            CXType_ConstantArray => TypeKind::ConstantArray,
            CXType_Vector => TypeKind::Vector,
            CXType_IncompleteArray => TypeKind::IncompleteArray,
            CXType_VariableArray => TypeKind::VariableArray,
            CXType_DependentSizedArray => TypeKind::DependentSizedArray,
            CXType_MemberPointer => TypeKind::MemberPointer,
            CXType_Auto => TypeKind::Auto,
            CXType_Elaborated => TypeKind::Elaborated,
            CXType_Pipe => TypeKind::Pipe,
            CXType_OCLImage1dRO => TypeKind::OCLImage1dRO,
            CXType_OCLImage1dArrayRO => TypeKind::OCLImage1dArrayRO,
            CXType_OCLImage1dBufferRO => TypeKind::OCLImage1dBufferRO,
            CXType_OCLImage2dRO => TypeKind::OCLImage2dRO,
            CXType_OCLImage2dArrayRO => TypeKind::OCLImage2dArrayRO,
            CXType_OCLImage2dDepthRO => TypeKind::OCLImage2dDepthRO,
            CXType_OCLImage2dArrayDepthRO => TypeKind::OCLImage2dArrayDepthRO,
            CXType_OCLImage2dMSAARO => TypeKind::OCLImage2dMSAARO,
            CXType_OCLImage2dArrayMSAARO => TypeKind::OCLImage2dArrayMSAARO,
            CXType_OCLImage2dMSAADepthRO => TypeKind::OCLImage2dMSAADepthRO,
            CXType_OCLImage2dArrayMSAADepthRO =>
                TypeKind::OCLImage2dArrayMSAADepthRO,
            CXType_OCLImage3dRO => TypeKind::OCLImage3dRO,
            CXType_OCLImage1dWO => TypeKind::OCLImage1dWO,
            CXType_OCLImage1dArrayWO => TypeKind::OCLImage1dArrayWO,
            CXType_OCLImage1dBufferWO => TypeKind::OCLImage1dBufferWO,
            CXType_OCLImage2dWO => TypeKind::OCLImage2dWO,
            CXType_OCLImage2dArrayWO => TypeKind::OCLImage2dArrayWO,
            CXType_OCLImage2dDepthWO => TypeKind::OCLImage2dDepthWO,
            CXType_OCLImage2dArrayDepthWO => TypeKind::OCLImage2dArrayDepthWO,
            CXType_OCLImage2dMSAAWO => TypeKind::OCLImage2dMSAAWO,
            CXType_OCLImage2dArrayMSAAWO => TypeKind::OCLImage2dArrayMSAAWO,
            CXType_OCLImage2dMSAADepthWO => TypeKind::OCLImage2dMSAADepthWO,
            CXType_OCLImage2dArrayMSAADepthWO =>
                TypeKind::OCLImage2dArrayMSAADepthWO,
            CXType_OCLImage3dWO => TypeKind::OCLImage3dWO,
            CXType_OCLImage1dRW => TypeKind::OCLImage1dRW,
            CXType_OCLImage1dArrayRW => TypeKind::OCLImage1dArrayRW,
            CXType_OCLImage1dBufferRW => TypeKind::OCLImage1dBufferRW,
            CXType_OCLImage2dRW => TypeKind::OCLImage2dRW,
            CXType_OCLImage2dArrayRW => TypeKind::OCLImage2dArrayRW,
            CXType_OCLImage2dDepthRW => TypeKind::OCLImage2dDepthRW,
            CXType_OCLImage2dArrayDepthRW => TypeKind::OCLImage2dArrayDepthRW,
            CXType_OCLImage2dMSAARW => TypeKind::OCLImage2dMSAARW,
            CXType_OCLImage2dArrayMSAARW => TypeKind::OCLImage2dArrayMSAARW,
            CXType_OCLImage2dMSAADepthRW => TypeKind::OCLImage2dMSAADepthRW,
            CXType_OCLImage2dArrayMSAADepthRW =>
                TypeKind::OCLImage2dArrayMSAADepthRW,
            CXType_OCLImage3dRW => TypeKind::OCLImage3dRW,
            CXType_OCLSampler => TypeKind::OCLSampler,
            CXType_OCLEvent => TypeKind::OCLEvent,
            CXType_OCLQueue => TypeKind::OCLQueue,
            CXType_OCLReserveID => TypeKind::OCLReserveID,
            CXType_ObjCObject => TypeKind::ObjCObject,
            CXType_ObjCTypeParam => TypeKind::ObjCTypeParam,
            CXType_Attributed => TypeKind::Attributed,
            _ => unreachable!(),
        }
    }

    pub fn typedef_name(&self) -> String {
        into_str(unsafe { clang_getTypedefName(self.t) })
    }

    pub fn spelling(&self) -> String {
        into_str(unsafe { clang_getTypeSpelling(self.t) })
    }

    pub fn canonical(&self) -> Ty {
        Ty {
            t: unsafe { clang_getCanonicalType(self.t) }
        }
    }

    pub fn pointee(&self) -> Ty {
        Ty {
            t: unsafe { clang_getPointeeType(self.t) }
        }
    }

    #[allow(non_upper_case_globals)]
    pub fn nullability(&self) -> Nullability {
        let ret = unsafe { clang_Type_getNullability(self.t) };
        match ret {
            CXTypeNullability_NonNull => Nullability::NonNull,
            CXTypeNullability_Nullable => Nullability::Nullable,
            CXTypeNullability_Unspecified => Nullability::Unspecified,
            CXTypeNullability_Invalid => Nullability::Unspecified,
            _ => panic!("Unexpected nullability"),
        }
    }

    pub fn invalid(&self) -> bool {
        self.t.kind == CXType_Invalid
    }

    pub fn is_const(&self) -> bool {
        unsafe { clang_isConstQualifiedType(self.t) != 0 }
    }

    pub fn is_variadic(&self) -> bool {
        unsafe { clang_isFunctionTypeVariadic(self.t) != 0 }
    }

    pub fn decl(&self) -> Cursor {
        Cursor {
            c: unsafe { clang_getTypeDeclaration(self.t) }
        }
    }

    pub fn modified_ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_Type_getModifiedType(self.t) }
        }
    }

    pub fn base_type(&self) -> Option<Ty> {
        let t = Ty {
            t: unsafe { clang_Type_getObjCObjectBaseType(self.t) }
        };
        if t.invalid() {
            None
        } else {
            Some(t)
        }
    }

    pub fn named_type(&self) -> Option<Ty> {
        let t = Ty {
            t: unsafe { clang_Type_getNamedType(self.t) }
        };
        if t.invalid() {
            None
        } else {
            Some(t)
        }
    }

    pub fn result_type(&self) -> Ty {
        Ty {
            t: unsafe { clang_getResultType(self.t) }
        }
    }

    pub fn num_args(&self) -> u32 {
        let num = unsafe { clang_getNumArgTypes(self.t) };
        if num < 0 {
            panic!("num_args called on wrong type");
        }
        num as u32
    }

    pub fn arg(&self, i: u32) -> Option<Ty> {
        let ty = Ty {
            t: unsafe { clang_getArgType(self.t, i) }
        };
        if ty.invalid() {
            None
        } else {
            Some(ty)
        }
    }

    pub fn function_arg_iter(&self) -> FunctionArgIterator {
        FunctionArgIterator {
            t: self,
            i: 0,
        }
    }

    pub fn element_ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_getArrayElementType(self.t) }
        }
    }

    pub fn array_size(&self) -> u64 {
        let size = unsafe { clang_getArraySize(self.t) };
        if size < 0 {
            panic!("Negative array size???");
        }
        size as u64
    }

    pub fn num_protocols(&self) -> u32 {
        unsafe { clang_Type_getNumObjCProtocolRefs(self.t) }
    }

    pub fn protocol_ref(&self, i: u32) -> Option<Cursor> {
        let cur = Cursor {
            c: unsafe { clang_Type_getObjCProtocolDecl(self.t, i) }
        };
        if cur.c.kind == CXCursor_NoDeclFound {
            None
        } else {
            Some(cur)
        }
    }

    pub fn protocol_ref_iter(&self) -> ObjCProtocolIterator {
        ObjCProtocolIterator {
            t: self,
            i: 0,
        }
    }

    pub fn num_type_args(&self) -> u32 {
        unsafe { clang_Type_getNumObjCTypeArgs(self.t) }
    }

    pub fn type_arg(&self, i: u32) -> Option<Ty> {
        let ty = Ty {
            t: unsafe { clang_Type_getObjCTypeArg(self.t, i) }
        };
        if ty.invalid() {
            None
        } else {
            Some(ty)
        }
    }

    pub fn type_arg_iter(&self) -> ObjCTypeArgIterator {
        ObjCTypeArgIterator {
            t: self,
            i: 0,
        }
    }
}

#[derive(Debug)]
pub enum Availability {
    Available,
    Deprecated(String),
    NotAvailable(String),
    NotAccessible,
}

#[derive(Debug)]
pub struct AvailabilityAttr {
    pub platform: String,
    pub introduced: CXVersion,
    pub deprecated: CXVersion,
    pub obsoleted: CXVersion,
    pub unavailable: bool,
    pub message: String,
}

impl AvailabilityAttr {
    unsafe fn from(a: &mut CXPlatformAvailability) -> AvailabilityAttr {
        let res = AvailabilityAttr {
            platform: into_str(a.Platform),
            introduced: a.Introduced,
            deprecated: a.Deprecated,
            obsoleted: a.Obsoleted,
            unavailable: a.Unavailable != 0,
            message: into_str(a.Message),
        };
        res
    }
}

pub struct Cursor {
    c: CXCursor,
}

impl Cursor {
    pub fn kind(&self) -> CursorKind {
        CursorKind::from_raw(self.c.kind)
    }

    pub fn name(&self) -> String {
        into_str(unsafe { clang_getCursorDisplayName(self.c) })
    }

    pub fn spelling(&self) -> String {
        into_str(unsafe { clang_getCursorSpelling(self.c) })
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation { loc: unsafe { clang_getCursorLocation(self.c) } }
    }

    pub fn property_attributes(&self) -> PropertyAttributes {
        PropertyAttributes {
            attr: unsafe { clang_Cursor_getObjCPropertyAttributes(self.c, 0) },
        }
    }

    pub fn getter_name(&self) -> String {
        into_str(unsafe { clang_Cursor_getObjCPropertyGetterName(self.c) })
    }

    pub fn setter_name(&self) -> String {
        into_str(unsafe { clang_Cursor_getObjCPropertySetterName(self.c) })
    }

    pub fn is_definition(&self) -> bool {
        unsafe { clang_isCursorDefinition(self.c) != 0 }
    }

    pub fn is_variadic(&self) -> bool {
        unsafe { clang_Cursor_isVariadic(self.c) != 0 }
    }

    pub fn availability(&self) -> Availability {
        let avail = unsafe { clang_getCursorAvailability(self.c) };
        match avail {
            CXAvailability_Available => Availability::Available,
            CXAvailability_Deprecated => unsafe {
                let mut msg = Default::default();
                clang_getCursorPlatformAvailability(
                    self.c,
                    ptr::null_mut(),
                    &mut msg as *mut _,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0
                );
                Availability::Deprecated(into_str(msg))
            },
            CXAvailability_NotAvailable => unsafe {
                let mut msg = Default::default();
                clang_getCursorPlatformAvailability(
                    self.c,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut msg as *mut _,
                    ptr::null_mut(),
                    0
                );
                Availability::NotAvailable(into_str(msg))
            },
            CXAvailability_NotAccessible => Availability::NotAccessible,
            _ => panic!("unexpected cursor availability {}", avail),
        }
    }

    pub fn availability_attrs(&self) -> Vec<AvailabilityAttr> {
        let mut buf: [CXPlatformAvailability; 8] = [Default::default(); 8];
        for avail in buf.iter_mut() {
            avail.Platform.data = ptr::null();
            avail.Message.data = ptr::null();
        }
        let len = unsafe {
            clang_getCursorPlatformAvailability(
                self.c,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                buf.as_mut_ptr(),
                buf.len() as i32
            )
        };
        buf.iter_mut().take(len as usize).map(|a| {
            unsafe { AvailabilityAttr::from(a) }
        }).collect()
    }

    pub fn num_args(&self) -> u32 {
        let len = unsafe { clang_Cursor_getNumArguments(self.c) };
        if len < 0 {
            panic!("num_args called on wrong cursor kind");
        }
        len as u32
    }

    pub fn arg(&self, i: u32) -> Cursor {
        Cursor {
            c: unsafe { clang_Cursor_getArgument(self.c, i) }
        }
    }

    pub fn arg_iter(&self) -> CursorArgIterator {
        CursorArgIterator {
            c: self,
            i: 0,
        }
    }

    pub fn ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_getCursorType(self.c) }
        }
    }

    pub fn typedef_ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_getTypedefDeclUnderlyingType(self.c) }
        }
    }

    pub fn result_ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_getCursorResultType(self.c) }
        }
    }

    pub fn enum_ty(&self) -> Ty {
        Ty {
            t: unsafe { clang_getEnumDeclIntegerType(self.c) }
        }
    }

    pub fn enum_const_value_signed(&self) -> i64 {
        unsafe { clang_getEnumConstantDeclValue(self.c) }
    }

    pub fn enum_const_value_unsigned(&self) -> u64 {
        unsafe { clang_getEnumConstantDeclUnsignedValue(self.c) }
    }

    pub fn visit_children<V>(&self, mut cb: V)
        where V: FnMut(Cursor) -> ChildVisit {
        unsafe {
            clang_visitChildren(
                self.c, visit_children::<V>, &mut cb as *mut _ as *mut _);
        }
    }
}

pub struct TranslationUnit<'a> {
    tu: CXTranslationUnit,
    p: PhantomData<&'a ()>,
}

extern "C" fn visit_children<V>(
    cur: CXCursor,
    _parent: CXCursor,
    data: CXClientData) -> CXChildVisitResult
    where V: FnMut(Cursor) -> ChildVisit
{
    let func: &mut V = unsafe { mem::transmute(data) };
    (*func)(Cursor { c: cur }) as CXChildVisitResult
}

impl<'a> TranslationUnit<'a> {
    pub fn visit<V>(&self, cb: V)
        where V: FnMut(Cursor) -> ChildVisit {
        let cur = Cursor {
            c: unsafe { clang_getTranslationUnitCursor(self.tu) }
        };
        cur.visit_children(cb);
    }
}

impl<'a> Drop for TranslationUnit<'a> {
    fn drop(&mut self) {
        unsafe {
            clang_disposeTranslationUnit(self.tu);
        }
    }
}

pub struct Index {
    idx: CXIndex,
}

impl Drop for Index {
    fn drop(&mut self) {
        unsafe {
            clang_disposeIndex(self.idx);
        }
    }
}

impl Index {
    pub fn new() -> Option<Index> {
        let idx = unsafe { clang_createIndex(0, 1) };
        if idx.is_null() {
            return None;
        }
        Some(Index {
            idx: idx,
        })
    }

    pub fn parse_tu(&self, args: &[&str], p: &Path) ->
        Option<TranslationUnit> {
        let cstrargs: Vec<_> = args.iter().map(|s| CString::new(s.as_bytes()).unwrap()).collect();
        let cargs: Vec<_> = cstrargs.iter().map(|s| s.as_bytes().as_ptr()).collect();
        let file = CString::new(p.to_str()?.as_bytes()).unwrap();
        let mut tu: CXTranslationUnit = ptr::null_mut();
        let ret = unsafe {
            clang_parseTranslationUnit2(
                self.idx,
                file.as_bytes().as_ptr() as *const _,
                cargs.as_ptr() as _, cargs.len() as i32,
                ptr::null_mut(), 0,
                CXTranslationUnit_IncludeAttributedTypes |
                CXTranslationUnit_VisitImplicitAttributes,
                &mut tu as *mut _)
        };
        if tu.is_null() {
            println!("Failed to parse tu. {}", ret);
            return None;
        }
        return Some(TranslationUnit {
            tu: tu,
            p: PhantomData,
        });
    }
}
