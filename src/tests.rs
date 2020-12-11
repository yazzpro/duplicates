
    use super::*;
    use mockall::predicate::*;
    
    
    #[test]
    fn test_d_deletes_all_but_1() {
        
        let mut f_mock = MockHandleFiles::new();        
        let mut d_mock = MockDataManager::new();
        
        f_mock.expect_remove_file().with(eq("1")).times(1).return_once(move |_x| Ok(()));
        f_mock.expect_remove_file().with(eq("2")).times(1).return_once(move |_x| Ok(()));

        d_mock.expect_delete_entry_for_path().with(eq("1")).times(1).return_once(move |_x| Ok(()));
        d_mock.expect_delete_entry_for_path().with(eq("2")).times(1).return_once(move |_x| Ok(()));

        delete(vec![String::from("1"),String::from("2"),String::from("3")], &f_mock, &d_mock);       
    }

    #[test]
    fn test_d_no_delete_if_only_1() {
        let f_mock = MockHandleFiles::new();        
        let d_mock = MockDataManager::new();
        delete(vec![String::from("1")], &f_mock, &d_mock);
    }

   #[test]
   fn test_the_same_entry_twice() {
	let f_mock = MockHandleFiles::new();
	let d_mock = MockDataManager::new();
	delete(vec![String::from("1"), String::from("1")], &f_mock, &d_mock);
   }

   #[test]
   fn test_empty_vector_dont_crash() {
	let f_mock = MockHandleFiles::new();
	let d_mock = MockDataManager::new();
	delete(vec![], &f_mock, &d_mock);
   }