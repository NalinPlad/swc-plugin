#![feature(core_intrinsics)]
use std::{any::Any, intrinsics::type_id};

use swc_core::{atoms::Atom, common::{Spanned, SyntaxContext}, ecma::{
    ast::{op, ExportDefaultExpr, Expr, Id, Ident, Pat, Program, VarDeclarator},
    transforms::testing::test_inline,
    visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
}};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};



// ScoutVisitor is used to find the unpredictable variable name

pub struct ScoutVisitor {
    pub export_name: Option<Atom>
}

// We implement Visit to access a read only AST
impl Visit for ScoutVisitor {
    fn visit_export_default_expr(&mut self,n: &swc_core::ecma::ast::ExportDefaultExpr) {
        if let Expr::Ident(ident) = &*n.expr {
            self.export_name = Some(ident.sym.clone());
        }
    }
}

// we need to have an empty impl to be able to call Fold directly on the Scout
impl VisitMut for ScoutVisitor {}



pub struct TransformVisitor {
    pub export_name: Atom
}

impl VisitMut for TransformVisitor {

    // Change the export name
    fn visit_mut_export_default_expr(&mut self,n: &mut ExportDefaultExpr) {
        n.expr = Box::new(Ident::new_no_ctxt("App".into(), n.expr.span()).into());
    }

    // Search for and change the declaration
    fn visit_mut_var_declarator(&mut self,n: &mut VarDeclarator) {
        if let Some(id) = &mut n.name.clone().ident() {
            if id.sym.eq(&self.export_name) {
                n.name = Ident::new_no_ctxt("App".into(), n.name.span()).into();
            }
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let mut scout = ScoutVisitor{ export_name: None };
    program.visit_with(&mut scout);

    // We didn't find a default export
    if let Some(export_name) = scout.export_name {
        let mut transform = TransformVisitor { export_name: export_name };
        program.fold_with(&mut as_folder(transform))
    } else {
        program.fold_with(&mut as_folder(scout))
    }

    // let mut transform = TransformVisitor { export_name: scout.export_name.unwrap_or("".into()) };
    // program.fold_with(&mut as_folder(v))

}

// test_inline!(
//     Default::default(),
//     |p| process_transform(p, p.comments.clone()),
//     export_default_inline,
//     // Input code
//     r#"let x = "some value"; export default x; console.log(App)"#,
//     // Output code after transformed with plugin
//     r#"let App = "some value"; export default App; console.log(App)"#
// );