use std::collections::HashMap;
use std::iter::zip;
use std::ops::DerefMut;

use anyhow::{anyhow, Context, Result};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_filesystem::db::FilesGroup;
use cairo_lang_filesystem::ids::{CrateId, CrateLongId};
use cairo_lang_starknet::contract::find_contracts;
use cairo_lang_starknet::contract_class::{compile_prepared_db, ContractClass};
use cairo_lang_utils::UpcastMut;
use scarb::compiler::helpers::build_compiler_config;
use scarb::compiler::{CompilationUnit, Compiler};
use scarb::core::Workspace;
use smol_str::SmolStr;
use starknet::core::types::contract::SierraClass;
use starknet::core::types::FieldElement;
use tracing::{trace, trace_span};

use crate::manifest::Manifest;

pub struct DojoCompiler;

impl Compiler for DojoCompiler {
    fn target_kind(&self) -> &str {
        "dojo"
    }

    fn compile(
        &self,
        unit: CompilationUnit,
        db: &mut RootDatabase,
        ws: &Workspace<'_>,
    ) -> Result<()> {
        let target_dir = unit.target_dir(ws.config());
        let compiler_config = build_compiler_config(&unit, ws);
        let main_crate_ids = collect_main_crate_ids(&unit, db);

        let contracts = {
            let _ = trace_span!("find_contracts").enter();
            find_contracts(db, &main_crate_ids)
        };

        trace!(
            contracts = ?contracts
                .iter()
                .map(|decl| decl.module_id().full_path(db.upcast_mut()))
                .collect::<Vec<_>>()
        );

        let contracts = contracts.iter().collect::<Vec<_>>();

        let classes = {
            let _ = trace_span!("compile_starknet").enter();
            compile_prepared_db(db, &contracts, compiler_config)?
        };

        // (contract name, class hash)
        let mut compiled_classes: HashMap<SmolStr, FieldElement> = HashMap::new();

        for (decl, class) in zip(contracts, classes) {
            let target_name = &unit.target().name;
            let contract_name = decl.submodule_id.name(db.upcast_mut());
            let file_name = format!("{target_name}_{contract_name}.json");

            let mut file = target_dir.open_rw(file_name.clone(), "output file", ws.config())?;
            serde_json::to_writer_pretty(file.deref_mut(), &class)
                .with_context(|| format!("failed to serialize contract: {contract_name}"))?;

            let class_hash = compute_class_hash_of_contract_class(class).with_context(|| {
                format!("problem computing class hash for contract `{contract_name}`")
            })?;
            compiled_classes.insert(contract_name, class_hash);
        }

        let mut file = target_dir.open_rw("manifest.json", "output file", ws.config())?;
        let manifest = Manifest::new(db, &main_crate_ids, compiled_classes);
        serde_json::to_writer_pretty(file.deref_mut(), &manifest)
            .with_context(|| "failed to serialize manifest")?;

        Ok(())
    }
}

fn compute_class_hash_of_contract_class(class: ContractClass) -> Result<FieldElement> {
    let class_str = serde_json::to_string(&class)?;
    let sierra_class = serde_json::from_str::<SierraClass>(&class_str)
        .map_err(|e| anyhow!("error parsing Sierra class: {e}"))?;
    sierra_class.class_hash().map_err(|e| anyhow!("problem hashing sierra contract: {e}"))
}

pub fn collect_main_crate_ids(unit: &CompilationUnit, db: &RootDatabase) -> Vec<CrateId> {
    let mut main_crate_ids = scarb::compiler::helpers::collect_main_crate_ids(unit, db);
    if unit.main_component().cairo_package_name() != "dojo" {
        main_crate_ids.push(db.intern_crate(CrateLongId("dojo".into())));
    }
    main_crate_ids
}

#[test]
fn test_compiler() {
    use dojo_test_utils::compiler::build_test_config;
    use scarb::ops;

    let config = build_test_config("../../examples/ecs/Scarb.toml").unwrap();
    let ws = ops::read_workspace(config.manifest_path(), &config)
        .unwrap_or_else(|op| panic!("Error building workspace: {op:?}"));
    ops::compile(&ws).unwrap_or_else(|op| panic!("Error compiling: {op:?}"))
}
