use std::path::Path;

use crate::modification::{add, create, create_diff, create_diff_builder, create_diff_from_destination_file, insert};
use crate::output::{CargoOutput, DirectoryStructure, SourceArchive};
use crate::stepper::Stepper;

pub fn step_all(
  destination_root_directory: impl AsRef<Path>,
  _use_local_pie_graph: bool,
  run_cargo: bool,
  create_outputs: bool,
) {
  let destination_root_directory = destination_root_directory.as_ref();
  let mut stepper = Stepper::new(
    "src",
    destination_root_directory,
    destination_root_directory,
    "src/gen/",
    run_cargo,
    ["build"],
    create_outputs,
  );

  // let pie_graph_path = PathBuf::from("../../graph");
  // // Use dunce to not make an absolute path prefixed with "\\?\" (UNC path) on Windows, as Cargo does not support these.
  // let pie_graph_path = dunce::canonicalize(pie_graph_path)
  //   .expect("failed to get absolute path to pie_graph");
  // let pie_graph_dependency = if use_local_pie_graph {
  //   format!("pie_graph = {{ path = '{}' }}", pie_graph_path.display())
  // } else {
  //   r#"pie_graph = "0.0.1""#.to_string()
  // };
  let pie_graph_dependency = r#"pie_graph = "0.0.1""#.to_string();
  stepper.add_substitution("%%%PIE_GRAPH_DEPENDENCY%%%", pie_graph_dependency);

  stepper.with_path("0_intro", |stepper| {
    stepper.with_path("1_setup", |stepper| {
      stepper
        .apply([
          add("Cargo_workspace.toml", "Cargo.toml"),
          add("Cargo.toml", "pie/Cargo.toml"),
          create("pie/src/lib.rs"),
        ])
        .output([
          DirectoryStructure::new(".", "dir.txt"),
          CargoOutput::new("cargo.txt"),
          SourceArchive::new("source.zip"),
        ]);
    });
  });

  stepper.with_path("1_programmability", |stepper| {
    stepper.with_path("1_api", |stepper| {
      stepper
        .apply([
          add("a_api.rs", "pie/src/lib.rs"),
        ])
        .output([
          CargoOutput::new("a_cargo.txt"),
          SourceArchive::new("source.zip"),
        ]);
    });
    stepper.with_path("2_non_incremental", |stepper| {
      stepper
        .apply([
          create_diff("a_context_module.rs", "pie/src/lib.rs"),
          add("b_non_incremental_module.rs", "pie/src/context/mod.rs"),
          create("pie/src/context/non_incremental.rs"),
        ])
        .output(DirectoryStructure::new(".", "b_dir.txt"));
      stepper.apply(add("c_non_incremental_context.rs", "pie/src/context/non_incremental.rs"));
      stepper.set_cargo_args(["test"]);
      stepper
        .apply(add("d_test.rs", "pie/src/context/non_incremental.rs"))
        .output(CargoOutput::new("d_cargo.txt"));
      stepper
        .apply_failure(create_diff("e_test_problematic.rs", "pie/src/context/non_incremental.rs"))
        .output(CargoOutput::new("e_cargo.txt"));
      stepper
        .apply_failure(create_diff("f_test_incompatible.rs", "pie/src/context/non_incremental.rs"))
        .output(CargoOutput::new("f_cargo.txt"));
      stepper.apply(create_diff("g_remove_test.rs", "pie/src/context/non_incremental.rs"));
      stepper.apply(create_diff("h_test_correct.rs", "pie/src/context/non_incremental.rs"))
        .output(SourceArchive::new("source.zip"));
    });
  });

  stepper.with_path("2_incrementality", |stepper| {
    stepper.with_path("1_require_file", |stepper| {
      stepper.apply([
        create_diff("a_context.rs", "pie/src/lib.rs"),
        create_diff("b_fs_module.rs", "pie/src/lib.rs"),
        add("c_fs.rs", "pie/src/fs.rs"),
        add("d_dev_shared_Cargo.toml", "dev_shared/Cargo.toml"),
        add("e_dev_shared_lib.rs", "dev_shared/src/lib.rs"),
        create_diff("e_Cargo_workspace.toml", "Cargo.toml"),
        create_diff("f_Cargo.toml", "pie/Cargo.toml"),
        add("g_fs_test.rs", "pie/src/fs.rs"),
        create_diff_builder("h_non_incremental_context.rs", "pie/src/context/non_incremental.rs")
          .original("../../1_programmability/2_non_incremental/c_non_incremental_context.rs") // HACK: Explicitly set original file to the one without tests
          .into_modification(),
      ])
        .output([
          DirectoryStructure::new(".", "e_dir.txt"),
          SourceArchive::new("source.zip"),
        ]);
    });
    stepper.with_path("2_stamp", |stepper| {
      stepper.apply([
        create_diff("a_module.rs", "pie/src/lib.rs"),
        add("b_file.rs", "pie/src/stamp.rs"),
        add("c_output.rs", "pie/src/stamp.rs"),
      ]);
      stepper.apply_may_fail([
        add("d1_test.rs", "pie/src/stamp.rs"),
      ]);
      stepper.apply([
        create_diff("d2_test_utilities.rs", "dev_shared/src/lib.rs"),
        create_diff("d3_test_correct.rs", "pie/src/stamp.rs"),
      ]);
      stepper.apply([
        create_diff("e_context_file.rs", "pie/src/lib.rs"),
        create_diff("f_context_task.rs", "pie/src/lib.rs"),
        create_diff("g_non_incremental_context.rs", "pie/src/context/non_incremental.rs"),
      ]).output(SourceArchive::new("source.zip"));
    });
    stepper.with_path("3_dependency", |stepper| {
      let dest = "pie/src/dependency.rs";
      stepper.apply([
        create_diff("a_module.rs", "pie/src/lib.rs"),
        add("b_file.rs", dest),
        add("c_task.rs", dest),
        add("d_dependency.rs", dest),
        add("e_test.rs", dest),
      ]).output(SourceArchive::new("source.zip"));
    });
    stepper.with_path("4_store", |stepper| {
      let dest = "pie/src/store.rs";
      stepper.apply([
        create_diff("a_Cargo.toml", "pie/Cargo.toml"),
        create_diff("b_module.rs", "pie/src/lib.rs"),
        add("c_basic.rs", dest),
        create_diff("d1_mapping_diff.rs", dest),
        create_diff("d2_mapping_diff.rs", dest),
        add("e_mapping.rs", dest),
        add("f_output.rs", dest),
        add("g_dependency.rs", dest),
        add("h_reset.rs", dest),
        add("i_test_file_mapping.rs", dest),
        insert("j_test_task_mapping.rs", "}", dest),
        insert("k_test_task_output.rs", "}", dest),
        insert("l_test_dependencies.rs", "}", dest),
        insert("m_test_reset.rs", "}", dest),
      ]).output(SourceArchive::new("source.zip"));
    });
    stepper.with_path("5_context", |stepper| {
      let dest = "pie/src/context/top_down.rs";
      stepper.apply([
        create_diff("a_module.rs", "pie/src/context/mod.rs"),
        add("b_basic.rs", dest),
        create_diff("c_current.rs", dest),
        create_diff("d_file.rs", dest),
        create_diff("e_task.rs", dest),
        create_diff("f_task_dep.rs", dest),
        create_diff("g_check.rs", dest),
        create_diff("h_error_field.rs", dest),
        create_diff("i_error_store.rs", dest),
      ]).output(SourceArchive::new("source.zip"));
    });
    stepper.with_path("6_example", |stepper| {
      let dest = "pie/examples/incremental.rs";
      stepper.set_cargo_args(["run", "--example", "incremental"]);
      stepper.apply([
        add("a_task.rs", dest),
        add("b_main.rs", dest),
      ]).output(CargoOutput::new("b_main.txt"));
      let insertion_place = "  Ok(())";
      stepper.apply([
        insert("c_reuse.rs", insertion_place, dest),
      ]).output(CargoOutput::new("c_reuse.txt"));
      stepper.apply([
        insert("d_file_dep.rs", insertion_place, dest),
        insert("e_diff_task.rs", insertion_place, dest),
        insert("f_diff_stamp.rs", insertion_place, dest),
      ]).output([
        CargoOutput::new("f_diff_stamp.txt"),
        SourceArchive::new("source.zip"),
      ]);
      stepper.set_cargo_args(["test"]);
    });
  });


  stepper.with_path("3_min_sound", |stepper| {
    stepper.with_path("1_session", |stepper| {
      stepper.set_cargo_args(["check"]);
      stepper.apply([
        create_diff("a_lib_import.rs", "pie/src/lib.rs"),
        add("b_lib_pie_session.rs", "pie/src/lib.rs"),
      ]);
      stepper.set_cargo_args(["check", "--lib"]);
      stepper.apply([
        create_diff("c_top_down_new.rs", "pie/src/context/top_down.rs"),
        create_diff("d_top_down_fix.rs", "pie/src/context/top_down.rs"),
        create_diff_from_destination_file("e_lib_require.rs", "pie/src/lib.rs"),
        create_diff_from_destination_file("f_lib_private_module.rs", "pie/src/lib.rs"),
      ]);
      stepper.set_cargo_args(["run", "--example", "incremental"]);
      stepper.apply([
        create_diff_from_destination_file("g_example.rs", "pie/examples/incremental.rs"),
      ]).output(SourceArchive::new("source.zip"));
      stepper.set_cargo_args(["test"]);
      stepper.apply([
        create_diff_from_destination_file("h_lib_consistent.rs", "pie/src/lib.rs"),
        create_diff_from_destination_file("i_context_consistent.rs", "pie/src/context/top_down.rs"),
      ]);
    });

    stepper.with_path("2_tracker", |stepper| {
      stepper.apply([
        create_diff("a_lib_module.rs", "pie/src/lib.rs"),
        add("b_tracker.rs", "pie/src/tracker/mod.rs"),
      ]);
      stepper.apply([
        add("c_noop.rs", "pie/src/tracker/mod.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("d_lib_tracker.rs", "pie/src/lib.rs"),
        create_diff_from_destination_file("e_top_down_tracker.rs", "pie/src/context/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_builder("f_mod_writing.rs", "pie/src/tracker/mod.rs")
          .original("b_tracker.rs")
          .into_modification(),
        add("g_writing.rs", "pie/src/tracker/writing.rs"),
        add("h_1_writing_impl.rs", "pie/src/tracker/writing.rs"),
        add("h_2_writing_impl.rs", "pie/src/tracker/writing.rs"),
      ]);
      stepper.set_cargo_args(["run", "--example", "incremental"]);
      stepper.apply([
        create_diff("i_writing_example.rs", "pie/examples/incremental.rs"),
      ]).output(CargoOutput::new("i_writing_example.txt"));
      stepper.set_cargo_args(["test"]);
      stepper.apply([
        create_diff_builder("j_mod_event.rs", "pie/src/tracker/mod.rs")
          .original("f_mod_writing.rs")
          .into_modification(),
        add("k_event.rs", "pie/src/tracker/event.rs"),
        add("l_event_tracker.rs", "pie/src/tracker/event.rs"),
        add("m_1_event_inspection.rs", "pie/src/tracker/event.rs"),
        add("m_2_event_inspection.rs", "pie/src/tracker/event.rs"),
      ]);
      stepper.apply([
        add("n_composite.rs", "pie/src/tracker/mod.rs"),
      ]).output(SourceArchive::new("source.zip"));
    });

    stepper.with_path("3_test", |stepper| {
      stepper.apply([
        add("a_1_common_pie.rs", "pie/tests/common/mod.rs"),
        add("a_2_common_ext.rs", "pie/tests/common/mod.rs"),
        add("a_3_common_task.rs", "pie/tests/common/mod.rs"),
      ]);
      stepper.apply([
        add("b_test_execute.rs", "pie/tests/top_down.rs")
      ]);
      stepper.apply([
        add("c_test_reuse.rs", "pie/tests/top_down.rs")
      ]);

      stepper.run_cargo(["test", "--", "--test-threads=1"], Some(true));
      stepper.run_cargo_applied(["test", "--test", "top_down", "test_reuse"], Some(true))
        .output(CargoOutput::new("c_test_reuse_stdout.txt"));

      stepper.apply([
        create_diff_from_destination_file("d_1_read_task.rs", "pie/tests/common/mod.rs"),
        create_diff_from_destination_file("d_2_test_require_file.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("e_1_lower_task.rs", "pie/tests/common/mod.rs"),
        create_diff_from_destination_file("e_2_test_require_task.rs", "pie/tests/top_down.rs"),
        create_diff_from_destination_file("e_3_test_require_task.rs", "pie/tests/top_down.rs"),
        create_diff_from_destination_file("e_4_test_require_task.rs", "pie/tests/top_down.rs"),
        create_diff_from_destination_file("e_5_test_require_task.rs", "pie/tests/top_down.rs"),
        create_diff_from_destination_file("e_6_test_require_task.rs", "pie/tests/top_down.rs"),
      ]).output(
        SourceArchive::new("source.zip")
      );
    });

    stepper.with_path("4_fix_task_dep", |stepper| {
      stepper.apply([
        create_diff_from_destination_file("a_upper_task.rs", "pie/tests/common/mod.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("b_test_setup.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.run_cargo(["test", "--test", "top_down", "test_no_superfluous_task_dependencies"], Some(true));
      stepper.apply_failure([
        create_diff_from_destination_file("c_test_manifest.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.run_cargo_applied(["test", "--test", "top_down", "test_no_superfluous_task_dependencies"], Some(false)).output([
        CargoOutput::with_modify_fn("c_test_manifest_2.txt", |log| log.split('🏁').nth(1).expect("second build to be in the build log").to_string()),
        CargoOutput::with_modify_fn("c_test_manifest_3.txt", |log| log.split('🏁').nth(2).expect("third build to be in the build log").to_string())
      ]);
      stepper.apply_failure([
        create_diff_from_destination_file("d_1_make_consistent.rs", "pie/src/context/top_down.rs"),
        create_diff_from_destination_file("d_2_task_dependency.rs", "pie/src/dependency.rs"),
        create_diff_from_destination_file("d_3_impl.rs", "pie/src/context/top_down.rs"),
        create_diff_from_destination_file("d_4_non_incremental.rs", "pie/src/context/non_incremental.rs"),
      ]);
      stepper.run_cargo_applied(["test", "--test", "top_down", "test_require_task"], Some(false)).output([
        CargoOutput::with_modify_fn("e_fix_tests_2.txt", |log| log.split('🏁').nth(1).expect("second build to be in the build log").to_string()),
        CargoOutput::with_modify_fn("e_fix_tests_3.txt", |log| log.split('🏁').nth(2).expect("third build to be in the build log").to_string())
      ]);
      stepper.apply([
        create_diff_from_destination_file("e_fix_tests.rs", "pie/tests/top_down.rs"),
      ]).output(
        SourceArchive::new("source.zip")
      );
    });

    stepper.with_path("5_overlap", |stepper| {
      stepper.apply([
        create_diff_from_destination_file("a_test_tasks.rs", "pie/tests/common/mod.rs"),
        create_diff_from_destination_file("b_test_issue.rs", "pie/tests/top_down.rs"),
        add("c_test_separate.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("d_dependency.rs", "pie/src/dependency.rs"),
        create_diff_from_destination_file("e_1_tracker.rs", "pie/src/tracker/mod.rs"),
        create_diff_from_destination_file("e_2_writing.rs", "pie/src/tracker/writing.rs"),
        create_diff_from_destination_file("e_3_event.rs", "pie/src/tracker/event.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("f_store.rs", "pie/src/store.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("g_context.rs", "pie/src/lib.rs"),
        create_diff_from_destination_file("h_non_incr.rs", "pie/src/context/non_incremental.rs"),
        create_diff_from_destination_file("i_top_down.rs", "pie/src/context/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("j_1_store.rs", "pie/src/store.rs"),
        create_diff_from_destination_file("j_2_top_down.rs", "pie/src/context/top_down.rs"),
      ]);
      stepper.apply_failure([
        create_diff_from_destination_file("k_1_use_provide.rs", "pie/tests/common/mod.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("k_2_fix_test.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("k_3_more_tests.rs", "pie/tests/top_down.rs"),
      ]).output(
        SourceArchive::new("source.zip")
      );
    });

    stepper.with_path("6_hidden_dep", |stepper| {
      stepper.apply([
        add("a_1_test.rs", "pie/tests/top_down.rs"),
        create_diff("a_2_test.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("b_1_store.rs", "pie/src/store.rs"),
        create_diff_from_destination_file("b_2_store.rs", "pie/src/store.rs"),
      ]);
      stepper.apply_failure(
        create_diff_from_destination_file("c_top_down.rs", "pie/src/context/top_down.rs")
      );
      stepper.apply([
        create_diff("d_1_test.rs", "pie/tests/top_down.rs"),
        add("d_2_test.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("e_1_read_origin.rs", "pie/tests/common/mod.rs"),
        create_diff_from_destination_file("e_2_read_refactor.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        add("f_1_test.rs", "pie/tests/top_down.rs"),
        create_diff("f_2_test.rs", "pie/tests/top_down.rs"),
        create_diff("f_3_test.rs", "pie/tests/top_down.rs"),
      ]).output(
        SourceArchive::new("source.zip")
      );
    });

    stepper.with_path("7_cycle", |stepper| {
      stepper.apply([
        create_diff_from_destination_file("a_task.rs", "pie/tests/common/mod.rs"),
      ]);
      stepper.apply_failure([
        add("b_test.rs", "pie/tests/top_down.rs"),
      ]);
      stepper.apply([
        create_diff_from_destination_file("c_1_dependency.rs", "pie/src/dependency.rs"),
        create_diff_from_destination_file("c_2_writing_tracker.rs", "pie/src/tracker/writing.rs"),
        create_diff_from_destination_file("c_3_store.rs", "pie/src/store.rs"),
        create_diff_from_destination_file("c_4_store_test.rs", "pie/src/store.rs"),
        create_diff_from_destination_file("c_5_top_down.rs", "pie/src/context/top_down.rs"),
      ]).output(
        SourceArchive::new("source.zip")
      );
    });
  });

  stepper.with_path("4_example", |stepper| {
    stepper.set_cargo_args(["run", "--example", "parser_dev"]);
    stepper.apply([
      create_diff_from_destination_file("a_1_Cargo.toml", "pie/Cargo.toml"),
      add("a_2_main.rs", "pie/examples/parser_dev/main.rs"),
    ]);

    stepper.set_cargo_args(["test", "--example", "parser_dev", "--", "--show-output"]);
    stepper.apply([
      create_diff_from_destination_file("a_3_main_parse_mod.rs", "pie/examples/parser_dev/main.rs"),
      add("a_4_grammar.rs", "pie/examples/parser_dev/parse.rs"),
      create_diff_from_destination_file("a_5_parse.rs", "pie/examples/parser_dev/parse.rs"),
      add("a_6_test.rs", "pie/examples/parser_dev/parse.rs"),
    ]);

    stepper.set_cargo_args(["build", "--example", "parser_dev"]);
    stepper.apply([
      create_diff_from_destination_file("b_1_main_task_mod.rs", "pie/examples/parser_dev/main.rs"),
      add("b_2_tasks_outputs.rs", "pie/examples/parser_dev/task.rs"),
      add("b_3_require_file.rs", "pie/examples/parser_dev/task.rs"),
      add("b_4_task.rs", "pie/examples/parser_dev/task.rs"),
    ]);

    stepper.set_cargo_args(["run", "--example", "parser_dev", "--", "--help"]);
    stepper.apply([
      create_diff_from_destination_file("c_1_Cargo.toml", "pie/Cargo.toml"),
      create_diff_from_destination_file("c_2_cli.rs", "pie/examples/parser_dev/main.rs"),
    ]);
    stepper.set_cargo_args(["run", "--example", "parser_dev", "--", "grammar.pest", "number", "test_1.txt", "test_2.txt"]);
    stepper.apply([
      create_diff_from_destination_file("c_3_compile_parse.rs", "pie/examples/parser_dev/main.rs"),
      add("c_4_grammar.pest", "grammar.pest"),
      add("c_4_test_1.txt", "test_1.txt"),
      add("c_4_test_2.txt", "test_2.txt"),
    ]);

    stepper.apply([
      create_diff_from_destination_file("d_1_Cargo.toml", "pie/Cargo.toml"),
      create_diff_from_destination_file("d_2_main_editor.rs", "pie/examples/parser_dev/main.rs"),
      add("d_3_editor.rs", "pie/examples/parser_dev/editor.rs"),
      create_diff_from_destination_file("d_4_main_cli.rs", "pie/examples/parser_dev/main.rs"),
    ]);
  });
}
