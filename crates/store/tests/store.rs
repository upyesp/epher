use epher_store::{DocStore, FunctionDoc, MemoryStore, Storage};

#[test]
fn function_round_trips_through_memory() {
    let store = DocStore::new(MemoryStore::default());
    assert!(store.get_function("f").unwrap().is_none());
    store
        .put_function(&FunctionDoc {
            name: "f".into(),
            source: "def f(x) = x ^ 2".into(),
        })
        .unwrap();
    let f = store.get_function("f").unwrap().unwrap();
    assert_eq!(f.name, "f");
    assert_eq!(f.source, "def f(x) = x ^ 2");
    assert_eq!(store.list_functions().unwrap().len(), 1);
}

#[test]
fn settings_round_trip() {
    let store = DocStore::new(MemoryStore::default());
    assert!(store.get_setting("language").unwrap().is_none());
    store
        .set_setting("language", serde_json::json!("fr"))
        .unwrap();
    assert_eq!(
        store.get_setting("language").unwrap(),
        Some(serde_json::json!("fr"))
    );
}

#[test]
fn overwrite_is_last_write_wins() {
    let store = DocStore::new(MemoryStore::default());
    store
        .set_setting("theme", serde_json::json!("dark"))
        .unwrap();
    store
        .set_setting("theme", serde_json::json!("light"))
        .unwrap();
    assert_eq!(
        store.get_setting("theme").unwrap(),
        Some(serde_json::json!("light"))
    );
}

#[test]
fn removal_removes() {
    let store = DocStore::new(MemoryStore::default());
    store
        .put_function(&FunctionDoc {
            name: "f".into(),
            source: "def f(x) = x".into(),
        })
        .unwrap();
    store.storage().remove("function/f").unwrap();
    assert!(store.get_function("f").unwrap().is_none());
}

#[cfg(feature = "fs")]
mod fs_tests {
    use super::*;
    use epher_store::FsStore;

    #[test]
    fn fs_store_round_trips_human_readable_files() {
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(FsStore::new(dir.path()));
        store
            .put_function(&FunctionDoc {
                name: "f".into(),
                source: "def f(x) = x ^ 2".into(),
            })
            .unwrap();
        assert_eq!(
            store.get_function("f").unwrap().unwrap().source,
            "def f(x) = x ^ 2"
        );
        // the file exists on disk as readable JSON
        let file = dir.path().join("function/f.json");
        assert!(file.exists(), "expected {file:?} to exist");
        let raw = std::fs::read_to_string(&file).unwrap();
        assert!(raw.contains("def f(x) = x ^ 2"));
    }

    #[test]
    fn fs_store_overwrites_and_lists() {
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(FsStore::new(dir.path()));
        store
            .put_function(&FunctionDoc {
                name: "f".into(),
                source: "def f(x) = x".into(),
            })
            .unwrap();
        store
            .put_function(&FunctionDoc {
                name: "g".into(),
                source: "def g(x) = x + 1".into(),
            })
            .unwrap();
        let names: Vec<String> = store
            .list_functions()
            .unwrap()
            .into_iter()
            .map(|d| d.name)
            .collect();
        assert_eq!(names, vec!["f", "g"]);
        store
            .set_setting("language", serde_json::json!("es"))
            .unwrap();
        // settings don't leak into functions
        assert_eq!(store.list_functions().unwrap().len(), 2);
    }
}

mod persist_tests {
    use super::*;
    use epher_core::Session;
    use epher_store::persist::{
        load_language, load_session, save_function, save_history, save_language, save_script,
    };

    #[test]
    fn load_session_restores_history_and_functions() {
        let store = DocStore::new(MemoryStore::default());
        save_history(&store, &["x = 1  = 1".to_string()]).unwrap();
        save_function(&store, "f", "def f(x) = x ^ 2").unwrap();
        save_script(&store, "setup", "y = 10").unwrap();
        let mut session = load_session(&store).unwrap();
        assert_eq!(session.history().len(), 1);
        // saved function and script are live in the session
        assert_eq!(session.submit("f(3)"), "= 9");
        assert_eq!(session.submit("y + 1"), "= 11");
    }

    #[test]
    fn history_saves_and_restores_round_trip() {
        let store = DocStore::new(MemoryStore::default());
        save_history(&store, &["a  = 1".to_string(), "b  = 2".to_string()]).unwrap();
        let session: Session = load_session(&store).unwrap();
        assert_eq!(
            session.history(),
            &["a  = 1".to_string(), "b  = 2".to_string()]
        );
    }

    #[test]
    fn language_setting_round_trips() {
        let store = DocStore::new(MemoryStore::default());
        assert_eq!(load_language(&store).unwrap(), None);
        save_language(&store, "fr").unwrap();
        assert_eq!(load_language(&store).unwrap(), Some("fr".to_string()));
    }
}

#[test]
fn replay_lines_lists_functions_then_scripts_in_load_order() {
    let store = DocStore::new(MemoryStore::default());
    epher_store::persist::save_script(&store, "later", "x = 2").unwrap();
    epher_store::persist::save_function(&store, "first", "def first() = 1").unwrap();
    let lines = epher_store::persist::replay_lines(&store).unwrap();
    assert_eq!(
        lines,
        vec!["def first() = 1".to_string(), "x = 2".to_string()]
    );
}
