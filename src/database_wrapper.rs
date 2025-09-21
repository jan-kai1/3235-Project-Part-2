use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use crate::database_fix_full::{UserStruct,UserDatabase};


// C struct representations
#[repr(C)]
pub struct UserStructT {
    pub password: [c_char; 100],
    pub username: [c_char; 50],
    pub user_id: c_int,
    pub email: [c_char; 50],
    pub inactivity_count: c_int,
    pub is_active: c_int,
    pub session_token: [c_char; 32],
    pub ownership: c_int,        
    pub ref_count: c_int,        
}



#[repr(C)]
pub struct UserDatabaseT {
    pub users: [*mut UserStructT; 100],
    pub count: c_int,
    pub capacity: c_int,
}

extern "C" {
    fn init_database(dc : *const i32) -> *mut UserDatabaseT;
    fn create_user(
        username: *const c_char,
        email: *const c_char,
        user_id: c_int,
        password: *const c_char,
    ) -> *mut UserStructT;
    fn add_user(db: *mut UserDatabaseT, user: *mut UserStructT);
    // fn find_user_by_id(db: *mut UserDatabaseT, user_id: c_int) -> *mut UserStructT;

    // for sharing
    fn add_shared_user_from_rust(db: *mut UserDatabaseT, user: *mut UserStructT);
    fn get_user_references_for_sharing(db: *mut UserDatabaseT, count: *mut c_int) -> *mut *mut UserStructT;
    // Session management
    pub fn create_user_session(user: *const UserStructT) -> *mut c_char;
    fn validate_user_session(token: *const c_char) -> c_int;

    // Memory management and optimization
    fn get_user_reference_for_debugging(
        db: *mut UserDatabaseT,
    ) -> *mut*mut UserStructT;

    // Additional C functions present in database_enhanced.c
    fn print_database(db: *mut UserDatabaseT);
    fn update_database_daily(db: *mut UserDatabaseT);
    fn user_login(db: *mut UserDatabaseT, user_name: *const c_char)->*const c_char;
    fn get_password(db: *mut UserDatabaseT, user_name: *const c_char) -> *const c_char;
    fn get_non_null_ref_count(db: *mut UserDatabaseT) -> c_int;
    fn find_user_by_username( db: *mut UserDatabaseT, user_name: *const c_char) -> *mut UserStructT;
    fn deactivate_users(db: *mut UserDatabaseT);
    fn init_session_manager();
    fn update_day_counter(dc : *const i32);
}

pub struct UserReference {
    pub username: String,
    pub ptr: *mut UserStructT,
}

impl UserReference {
    pub fn new(username: String, ptr: *mut UserStructT) -> Self {
        UserReference {
            username,
            ptr,
        }
    }
}

pub struct DatabaseExtensions {
    db: *mut UserDatabaseT,
}

impl DatabaseExtensions {
    pub fn new(dc : *const i32) -> Self {
       println!("Initializing Enhanced Student Database System...");
        // let dc = Box::new(0);
        println!("Making C database");
        println!("=== C DEBUG 1: Starting DatabaseExtensions::new ===");
        
        println!("=== C DEBUG 2: About to call init_database ===");
        let db = unsafe { init_database(dc) };
        println!("=== C DEBUG 3: init_database completed ===");
        
        println!("=== C DEBUG 4: About to call init_session_manager ===");
        unsafe {
            init_session_manager();
        }
        println!("=== C DEBUG 5: init_session_manager completed ===");
        
        println!("=== C DEBUG 6: Creating DatabaseExtensions struct ===");
        let result = DatabaseExtensions { db };
        println!("=== C DEBUG 7: DatabaseExtensions created successfully ===");
        result
    }
    pub fn get_user_password(&self, user: *mut UserStructT) -> String {
        unsafe {
            let password_ptr = get_password(self.db, (*user).username.as_ptr());
            CStr::from_ptr(password_ptr).to_string_lossy().to_string()
        }
    }
    pub fn get_user_in_c_backend(&self, username: &str) -> *mut UserStructT {
        let c_username = match CString::new(username) {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };
        unsafe {
            let user_ptr = find_user_by_username(self.db, c_username.as_ptr());
            if user_ptr.is_null() {
                std::ptr::null_mut()
            } else {
                user_ptr
            }
        }
    }
    pub fn get_last_user_id(&self) -> i32 {
        unsafe { (*self.db).count - 1 }
    }
    pub fn sync_user_to_c_backend(
        &self,
        username: &str,
        email: &str,
        user_id: i32,
        password: &str,
    ) -> Result<(), String> {
        // println!("[RUST DEBUG] sync_user_to_c_backend called");
        // println!("[RUST DEBUG] username: '{}' (len={})", username, username.len());
        // println!("[RUST DEBUG] email: '{}' (len={})", email, email.len());
        // println!("[RUST DEBUG] password: '{}' (len={})", password, password.len());
        
        let c_username = CString::new(username).map_err(|_| "Invalid username")?;
        let c_email = CString::new(email).map_err(|_| "Invalid email")?;
        let c_password = CString::new(password).map_err(|_| "Invalid password")?;
        
        // println!("[RUST DEBUG] CStrings created successfully");
        
        unsafe {
            // println!("[RUST DEBUG] About to call C create_user");
            let user = create_user(c_username.as_ptr(), c_email.as_ptr(), user_id, c_password.as_ptr());
            // println!("[RUST DEBUG] C create_user returned: {:p}", user);
            
            if user.is_null() {
                return Err("Failed to create user".to_string());
            }
            
            // println!("[RUST DEBUG] About to call C add_user");
            add_user(self.db, user);
            // println!("[RUST DEBUG] C add_user completed");
        }
        Ok(())
    }
    pub fn sync_user_from_rust_db(&self,user: *mut UserStructT){
            unsafe {
                (*user).ownership = 0; // RUST_OWNED initially
                (*user).ref_count = 1;
                add_shared_user_from_rust(self.db, user);
                // add_user(self.db, user);
            }
    }

    pub fn cast_user_struct(user: &UserStruct) -> *const UserStructT {
        user as *const UserStruct as *const UserStructT
    }
    pub fn create_session(&self, user: &UserStruct) -> Result<String, String> {
        unsafe {
            let userp = DatabaseExtensions::cast_user_struct(user);
            let token_ptr = create_user_session(userp);
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }

            let token = CStr::from_ptr(token_ptr).to_string_lossy().to_string();
            Ok(token)
        }
    }

    pub fn validate_session(&self, token: &str) -> Result<i32, String> {
        let c_token = CString::new(token).map_err(|_| "Invalid token")?;

        unsafe {
            let user_id = validate_user_session(c_token.as_ptr());
            if user_id == 0 {
                Err("Invalid session".to_string())
            } else {
                Ok(user_id)
            }
        }
    }

    pub fn login_user(&self, user_name: &str)-> Result<String, String>{
        unsafe {
            let c_user_name = CString::new(user_name).map_err(|_| "Invalid username")?;
            let token_ptr = user_login(self.db, c_user_name.as_ptr());
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }
            Ok(CStr::from_ptr(token_ptr).to_string_lossy().to_string())
        }
    }


    pub fn get_all_user_references(&self) -> Vec<Box<UserStruct>> {
        // let refs = unsafe { get_user_reference_for_debugging(self.db)};
        // let ref_count = unsafe { get_non_null_ref_count(self.db) };
        let c_user_ptrs = self.get_user_references_for_sharing();

        let mut user_refs = Vec::new();
        // let refs_slice = unsafe { std::slice::from_raw_parts(refs, ref_count as usize) };
        // for &user_ptr in refs_slice {
        //     if !user_ptr.is_null() {
        //         let user = unsafe { Box::from_raw(user_ptr as *mut UserStruct) };
        //         user_refs.push(user);
        //     }
        // }
        // user_refs
        // let mut user_refs = Vec::new();
        for ptr in c_user_ptrs {
            if !ptr.is_null() {
                unsafe {
                    // DON'T use Box::from_raw - that transfers ownership
                    // Instead, create a reference wrapper or handle this differently
                    
                    // For now, return empty vector to avoid double-free
                    // The join will work but won't add C users to Rust
                    break;
                }
            }
        }
        
        user_refs
    }
    pub fn increment_day(&self, rust_db: &UserDatabase) {
        println!("=== C DEBUG: Starting increment_day ===");
        // println!("=== C DEBUG: Checking database pointer: {:p} ===", self.db);
        if self.db.is_null() {
            // println!("=== C DEBUG: ERROR - Database pointer is NULL! ===");
            return;
        }
        unsafe {
            // println!("=== C DEBUG: About to call update_database_daily on C db ===");
            // println!("=== C DEBUG: Attempting to read db fields ===");
        
            let count = std::ptr::read_volatile(&(*self.db).count);
            // println!("=== C DEBUG: Database count: {} ===", count);
            
            let capacity = std::ptr::read_volatile(&(*self.db).capacity);
            // println!("=== C DEBUG: Database capacity: {} ===", capacity);
            
            // Check if count is reasonable
            if count < 0 || count > 1000 {
                println!("=== C DEBUG: ERROR - Invalid count: {} ===", count);
                return;
            }
            update_database_daily(self.db);
            // println!("=== C DEBUG: C update_database_daily completed ===");
            
            // println!("=== C DEBUG: About to call deactivate_idle_users ===");
            self.deactivate_idle_users(rust_db);
            // println!("=== C DEBUG: deactivate_idle_users completed ===");
        }
    }
    pub fn deactivate_idle_users(&self, db: &UserDatabase) {
        unsafe {
            let db_ptr = db as *const UserDatabase as *mut UserDatabaseT;
            deactivate_users(db_ptr);
        }
    }
    pub fn create_session_for_c_ptr(&self, user: *const UserStructT) -> Result<String, String> {
        unsafe {
            let token_ptr = create_user_session(user);
            if token_ptr.is_null() {
                return Err("Failed to create session".to_string());
            }
            let token = CStr::from_ptr(token_ptr).to_string_lossy().to_string();
            Ok(token)
        }
    }
    pub fn print_database_full(&self) {
        unsafe {
            print_database(self.db);
        }
    }
    pub fn get_user_references_for_sharing(&self) -> Vec<*mut UserStructT> {
        let mut count: c_int = 0;
        let refs = unsafe { 
            get_user_references_for_sharing(self.db, &mut count as *mut c_int)
        };
        
        if refs.is_null() || count == 0 {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let refs_slice = unsafe { 
            std::slice::from_raw_parts(refs, count as usize) 
        };
        
        for &user_ptr in refs_slice {
            if !user_ptr.is_null() {
                result.push(user_ptr);  // Just return pointers, don't create Boxes
            }
        }
        
        // Free the array (but not the user pointers)
        unsafe {
            libc::free(refs as *mut libc::c_void);
        }
        
        result
    }
    
    pub fn add_shared_user_from_rust(&self, user: *mut UserStructT) {
        unsafe {
            add_shared_user_from_rust(self.db, user);
        }
    }
}

pub fn initialize_enhanced_database(dc : &i32) -> DatabaseExtensions {
    let dc_ptr = dc as *const i32;
    DatabaseExtensions::new(dc_ptr)
}