
    use super::*;
    use counters::Counters;
    use counters::filters::*;

    struct MockFileManager{
         items_deleted : Counters
    }
    impl MockFileManager {
        
        pub fn d(&self) {
            self.items_deleted.event("D");
        }
        pub fn count(&self) -> u64 {
            self.items_deleted.accumulate(Contains("D"))
        }
    }
    impl HandleFiles for MockFileManager {
        fn remove_file(&self, _path: &str) -> std::io::Result<()>{            
            self.d();
            Ok(())
        }
        fn get_full_path(&self, srcdir: &PathBuf) -> std::io::Result<PathBuf>{
            let p = srcdir.to_owned();
            Ok(p)
        }
        fn walkdir(&self, _srcdir: &str) -> walkdir::IntoIter{
            walkdir::WalkDir::new("this_dir_doesnt_exist").into_iter()
        }
        fn get_file(&self, _path: &PathBuf) -> std::io::Result<File> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
        } 
    }

    struct MockDb {
        
    }
    impl DataManager for MockDb {
        fn create_tables(&self) -> rusqlite::Result<()> { Ok(())}
        fn get_entries_by_hash(&self,_hash: &str) -> rusqlite::Result<Vec<FileInfo>> { Ok(vec![])}
        fn get_entry_for_path(&self,_path: &str) -> rusqlite::Result<Option<FileInfo>> { Ok(None)}
        fn delete_entry_for_path(&self,_path: &str) -> rusqlite::Result<()>{ Ok(())}
        fn add_entry(&self,_entry: &FileInfo) -> rusqlite::Result<()>{ Ok(())}    
    }


    #[test]
    fn test_config_d_deletes_all_but_1() {
        
        let f_mock = MockFileManager{
            items_deleted: Counters::new()
        };
        let d_mock = MockDb{};
        delete(vec![String::from("1"),String::from("2"),String::from("3")], &f_mock, &d_mock);

        assert_eq!(2, f_mock.count());

    }

    #[test]
    fn test_config_d_no_delete_if_only_1() {
        
        let f_mock = MockFileManager{
            items_deleted: Counters::new()
        };
        let d_mock = MockDb{};
        delete(vec![String::from("1")], &f_mock, &d_mock);

        assert_eq!(0, f_mock.count());

    }

