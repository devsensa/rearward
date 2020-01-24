#[cfg(test)]
mod tests {
    use rkv::{Manager, MultiStore, Rkv, SingleStore, StoreError, StoreOptions, Value};
    use std::fs;
    use std::str;
    use std::sync::RwLockReadGuard;
    use tempfile::Builder;

    #[test]
    fn test_rkv() {
        let root = Builder::new().prefix("test_db").tempdir().unwrap();
        fs::create_dir_all(root.path()).unwrap();
        let path = root.path();
        let create_arc = Manager::singleton()
            .write()
            .unwrap()
            .get_or_create(path, Rkv::new)
            .unwrap();
        let env = create_arc.read().unwrap();
        let store = env.open_single("mydb", StoreOptions::create()).unwrap();
        {
            // Use a write transaction to mutate the store via a `Writer`.
            // There can be only one writer for a given environment, so opening
            // a second one will block until the first completes.
            let mut writer = env.write().unwrap();

            // Keys are `AsRef<[u8]>`, while values are `Value` enum instances.
            // Use the `Blob` variant to store arbitrary collections of bytes.
            // Putting data returns a `Result<(), StoreError>`, where StoreError
            // is an enum identifying the reason for a failure.
            store.put(&mut writer, "int", &Value::I64(1234)).unwrap();
            store
                .put(&mut writer, "uint", &Value::U64(1234_u64))
                .unwrap();
            store
                .put(&mut writer, "float", &Value::F64(1234.0.into()))
                .unwrap();
            store
                .put(&mut writer, "instant", &Value::Instant(1528318073700))
                .unwrap();
            store
                .put(&mut writer, "boolean", &Value::Bool(true))
                .unwrap();
            store
                .put(&mut writer, "string", &Value::Str("Héllo, wörld!"))
                .unwrap();
            store
                .put(
                    &mut writer,
                    "json",
                    &Value::Json(r#"{"foo":"bar", "number": 1}"#),
                )
                .unwrap();
            store
                .put(&mut writer, "blob", &Value::Blob(b"blob"))
                .unwrap();

            // You must commit a write transaction before the writer goes out
            // of scope, or the transaction will abort and the data won't persist.
            writer.commit().unwrap();
        }
        {
            // Use a read transaction to query the store via a `Reader`.
            // There can be multiple concurrent readers for a store, and readers
            // never block on a writer nor other readers.
            let reader = env.read().expect("reader");

            // Keys are `AsRef<u8>`, and the return value is `Result<Option<Value>, StoreError>`.
            println!("Get int {:?}", store.get(&reader, "int").unwrap());
            println!("Get uint {:?}", store.get(&reader, "uint").unwrap());
            println!("Get float {:?}", store.get(&reader, "float").unwrap());
            println!("Get instant {:?}", store.get(&reader, "instant").unwrap());
            println!("Get boolean {:?}", store.get(&reader, "boolean").unwrap());
            println!("Get string {:?}", store.get(&reader, "string").unwrap());
            println!("Get json {:?}", store.get(&reader, "json").unwrap());
            println!("Get blob {:?}", store.get(&reader, "blob").unwrap());

            // Retrieving a non-existent value returns `Ok(None)`.
            println!(
                "Get non-existent value {:?}",
                store.get(&reader, "non-existent").unwrap()
            );

            // A read transaction will automatically close once the reader
            // goes out of scope, so isn't necessary to close it explicitly,
            // although you can do so by calling `Reader.abort()`.
        }

        {
            // Aborting a write transaction rolls back the change(s).
            let mut writer = env.write().unwrap();
            store.put(&mut writer, "foo", &Value::Str("bar")).unwrap();
            writer.abort();
            let reader = env.read().expect("reader");
            println!(
                "It should be None! ({:?})",
                store.get(&reader, "foo").unwrap()
            );
        }

        {
            // Explicitly aborting a transaction is not required unless an early
            // abort is desired, since both read and write transactions will
            // implicitly be aborted once they go out of scope.
            {
                let mut writer = env.write().unwrap();
                store.put(&mut writer, "foo", &Value::Str("bar")).unwrap();
            }
            let reader = env.read().expect("reader");
            println!(
                "It should be None! ({:?})",
                store.get(&reader, "foo").unwrap()
            );
        }

        {
            // Deleting a key/value pair also requires a write transaction.
            let mut writer = env.write().unwrap();
            store.put(&mut writer, "foo", &Value::Str("bar")).unwrap();
            store.put(&mut writer, "bar", &Value::Str("baz")).unwrap();
            store.delete(&mut writer, "foo").unwrap();

            // A write transaction also supports reading, and the version of the
            // store that it reads includes the changes it has made regardless of
            // the commit state of that transaction.
            // In the code above, "foo" and "bar" were put into the store,
            // then "foo" was deleted so only "bar" will return a result when the
            // database is queried via the writer.
            println!(
                "It should be None! ({:?})",
                store.get(&writer, "foo").unwrap()
            );
            println!("Get bar ({:?})", store.get(&writer, "bar").unwrap());

            // But a reader won't see that change until the write transaction
            // is committed.
            {
                let reader = env.read().expect("reader");
                println!("Get foo {:?}", store.get(&reader, "foo").unwrap());
                println!("Get bar {:?}", store.get(&reader, "bar").unwrap());
            }
            writer.commit().unwrap();
            {
                let reader = env.read().expect("reader");
                println!(
                    "It should be None! ({:?})",
                    store.get(&reader, "foo").unwrap()
                );
                println!("Get bar {:?}", store.get(&reader, "bar").unwrap());
            }

            // Committing a transaction consumes the writer, preventing you
            // from reusing it by failing at compile time with an error.
            // This line would report error[E0382]: borrow of moved value: `writer`.
            // store.put(&mut writer, "baz", &Value::Str("buz")).unwrap();
        }

        {
            // Clearing all the entries in the store with a write transaction.
            {
                let mut writer = env.write().unwrap();
                store.put(&mut writer, "foo", &Value::Str("bar")).unwrap();
                store.put(&mut writer, "bar", &Value::Str("baz")).unwrap();
                writer.commit().unwrap();
            }

            {
                let mut writer = env.write().unwrap();
                store.clear(&mut writer).unwrap();
                writer.commit().unwrap();
            }

            {
                let reader = env.read().expect("reader");
                println!(
                    "It should be None! ({:?})",
                    store.get(&reader, "foo").unwrap()
                );
                println!(
                    "It should be None! ({:?})",
                    store.get(&reader, "bar").unwrap()
                );
            }
        }
    }

    #[test]
    fn test_rkv_iterators() {
        let root = Builder::new().prefix("iterator").tempdir().unwrap();
        fs::create_dir_all(root.path()).unwrap();
        let p = root.path();

        let created_arc = Manager::singleton()
            .write()
            .unwrap()
            .get_or_create(p, Rkv::new)
            .unwrap();
        let k = created_arc.read().unwrap();
        let store = k.open_single("store", StoreOptions::create()).unwrap();

        populate_store(&k, store).unwrap();

        let reader = k.read().unwrap();

        println!("Iterating from the beginning...");
        // Reader::iter_start() iterates from the first item in the store, and
        // returns the (key, value) tuples in order.
        let mut iter = store.iter_start(&reader).unwrap();
        while let Some(Ok((country, city))) = iter.next() {
            println!("{}, {:?}", str::from_utf8(country).unwrap(), city);
        }

        println!();
        println!("Iterating from the given key...");
        // Reader::iter_from() iterates from the first key equal to or greater
        // than the given key.
        let mut iter = store.iter_from(&reader, "Japan").unwrap();
        while let Some(Ok((country, city))) = iter.next() {
            println!("{}, {:?}", str::from_utf8(country).unwrap(), city);
        }

        println!();
        println!("Iterating from the given prefix...");
        let mut iter = store.iter_from(&reader, "United Kin").unwrap();
        while let Some(Ok((country, city))) = iter.next() {
            println!("{}, {:?}", str::from_utf8(country).unwrap(), city);
        }
        println!("Iterating from the given prefix finished");

        fn populate_store(k: &Rkv, store: SingleStore) -> Result<(), StoreError> {
            let mut writer = k.write()?;
            for (country, city) in vec![
                ("Canada", Value::Str("Ottawa")),
                ("United_States of America", Value::Str("Washington")),
                ("Germany", Value::Str("Berlin")),
                ("France", Value::Str("Paris")),
                ("Italy", Value::Str("Rome")),
                ("United_Kingdom", Value::Str("London")),
                ("Japan", Value::Str("Tokyo")),
            ] {
                store.put(&mut writer, country, &city)?;
            }
            writer.commit()
        }
    }
}
