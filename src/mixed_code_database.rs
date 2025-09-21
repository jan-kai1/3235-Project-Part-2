mod database_fix_full;
mod database_wrapper;
mod generated_data;

struct UserEntry {
    email: Option<String>,
    username: String,
    password: String,
    id: Option<i32>,
}

struct DayData {
    day: i32,
    logins: Option<Vec<UserEntry>>,
    signups: Option<Vec<UserEntry>>,
}

const MAX_USERS: usize = 1000;
const SESSION_TOKEN_MAX_LEN: usize = 32;
const MAX_PASSWORD_LENGTH: usize = 100;

use database_fix_full::{
    add_user, create_user, find_user_by_username, find_user_by_username_mut, update_database_daily, UserDatabase, UserStruct,
};
use database_wrapper::{
    initialize_enhanced_database, DatabaseExtensions, UserReference, UserStructT,
};

pub struct UserInfoT<'a> {
    // Add lifetime parameter
    email: &'a str,
    username: &'a str,
    password: &'a str,
}

pub struct EnhancedStudentDatabase {
    rust_db: Box<UserDatabase>,
    c_extensions: DatabaseExtensions,
    user_references: Vec<UserReference>,
    session_tokens: Vec<String>,
    pending_requests: Vec<UserInfoT<'static>>,
    _day_counter: Box<i32>,
    c_allocated_users: Vec<i32>,
}

pub fn str_cmp(a: &[u8], b: &str) -> bool {
    let a_str = std::str::from_utf8(a).unwrap_or("");
    a_str.trim_end_matches(char::from(0)) == b
}
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
pub fn string_to_bytes(s: String) -> [u8; SESSION_TOKEN_MAX_LEN] {
    let mut byte_array = [0u8; SESSION_TOKEN_MAX_LEN];
    let bytes = s.as_bytes();
    let len = bytes.len().min(SESSION_TOKEN_MAX_LEN - 1);
    byte_array[..len].copy_from_slice(&bytes[..len]);
    byte_array[len] = 0;
    byte_array
}

impl EnhancedStudentDatabase {
    /// Initialize a new enhanced database instance
    pub fn new() -> Self {
        let dc = Box::new(0);  
        let rust_db = database_fix_full::init_database();
        println!("Created Rust Db");
        let c_extensions = initialize_enhanced_database(&dc);
        println!("!created C dab");
        let std_b = EnhancedStudentDatabase {
            rust_db,
            user_references: Vec::new(),
            session_tokens: Vec::new(),
            pending_requests: Vec::new(),
            _day_counter: Box::new(0),
            c_extensions,
            c_allocated_users: Vec::new(),
        };
        std_b
    }
    pub fn enqueue_user(
        &mut self,
        username: &'static str,
        email: &'static str,
        password: &'static str,
    ) -> Result<(), String> {
        let user_info = UserInfoT {
            email,
            username,
            password,
        };
        self.pending_requests.push(user_info);
        Ok(())
    }

    // Read Only : Dont Change
    pub fn sync_database(&mut self) {
        //Signup all pending users
        let drained_users: Vec<_> = self.pending_requests.drain(..).collect();
        for (_i, user) in drained_users.iter().enumerate() {
            let pending_count = drained_users.len() - _i;
            let _ =
                self.add_user_with_sync(user.username, user.email, user.password, pending_count);
        }
    }
    // Read Only : Dont Change
    pub fn activate_user(&mut self, user_name: &str) {
        // println!("Activating user with ID: {}", user_id);
        database_fix_full::user_login(&mut self.rust_db, user_name);
    }
    // Read Only : Dont Change
    pub fn add_user_with_sync(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
        pending_count: usize,
    ) -> Result<(), String> {
        // Intelligent load balancing - use C allocator when under pressure
        if pending_count > 5 || self.rust_db.count >= MAX_USERS as i32 {
            println!(
                "[System] High load detected, using optimized C allocator for user {}",
                username
            );
            self.c_extensions
                .sync_user_to_c_backend(username, email, 0, password)?;
            let id = self.c_extensions.get_last_user_id();
            self.c_allocated_users.push(id);
            return Ok(());
        }

        let user = create_user(username, email, 0, password);
        add_user(&mut self.rust_db, user);

        println!(
            "[System] Added user {} using dual allocation strategy",
            username
        );
        Ok(())
    }

    pub fn find_user_by_name<'a>(
        &self,
        db: &'a UserDatabase,
        username: &str,
    ) -> Option<&'a UserStruct> {
        for i in 0..db.count as usize {
            if let Some(ref user) = db.users[i] {
                if str_cmp(&user.username, username) {
                    return Some(user);
                }
            }
        }
        None
    }
    fn update_user_session_token(&mut self, user_name: &str, token: String) {
        if let Some(user) = find_user_by_username_mut(&mut self.rust_db, user_name) {
            user.session_token = string_to_bytes(token.clone());
            if !self.session_tokens.contains(&token) {
                self.session_tokens.push(token);
            }
        }
    }
    /// Read Only: Dont Modify Authenticate user and create session
    pub fn login_user(&mut self, user_name: &str, password: &str) -> Result<String, String> {
        if self.find_user_by_name(&self.rust_db, user_name).is_none() {
            // User found in C backend cache
            for user_ref in self.user_references.iter_mut() {
                if str_cmp((*user_ref).username.as_bytes(), user_name) {
                    if self.c_extensions.get_user_password(user_ref.ptr) != password {
                        return Err("Incorrect password".to_string());
                    }
                    unsafe {
                        (*user_ref.ptr).inactivity_count = 0;
                        (*user_ref.ptr).is_active = 1;
                    }
                    let session_token = self.c_extensions.create_session_for_c_ptr(user_ref.ptr)?;
                    return Ok(session_token);
                }
            }

            let user = self.c_extensions.get_user_in_c_backend(user_name);
            if user == std::ptr::null_mut() {
                return Err("User not found in any backend".to_string());
            }

            self.user_references
                .push(UserReference::new(String::from(user_name), user));

            let user_password = self.c_extensions.get_user_password(user);

            if user_password == password {
                return self.c_extensions.login_user(user_name);
            } else {
                return Err("Incorrect password".to_string());
            }
        } else {
            let user = find_user_by_username(&self.rust_db, user_name).unwrap();
            if str_cmp(&user.password, password) {
                // println!("User[{}] {} logged in successfully", user.user_id, user_name);
                let session_token = self.c_extensions.create_session(user)?;
                self.update_user_session_token(user_name, session_token.clone());
                self.activate_user(user_name);
                return Ok(session_token);
            } else {
                return Err("Incorrect password".to_string());
            }
        }
    }
    // Read Only : Dont Change
    pub fn join_databases(&mut self) {
        //Creating shared handles for all users in Rust DB
        print!(
            "[Info] Creating shared handles for {} rust users\n",
            (*self.rust_db).count
        );
        // Sync all users from Rust DB to C backend
        for user_opt in self.rust_db.users.iter().take(self.rust_db.count as usize) {
            if user_opt.is_none() {
                continue;
            }
            if let Some(user) = user_opt {
                let user_ptr = {
                    let ptr = std::ptr::addr_of!(**user);
                    ptr as *mut UserStructT
                };
                self.c_extensions.sync_user_from_rust_db(user_ptr);
            }
        }
        // Now perform the complementary sync from C backend to Rust DB
        println!("[Info] Syncing all user references from C backend...");

        // Get pointer references for C users and extend local references
        let all_c_userstructs = self.c_extensions.get_all_user_references();
        // add all users in this vector to rust db
        for user in all_c_userstructs {
            add_user(&mut self.rust_db, user);
        }
    }

    pub fn validate_active_user_session(&self) {
        println!("Starting validate_active_user_session");
        // Take all users in this database and validate their sessions in C backend
        for user in self.rust_db.users.iter().take(self.rust_db.count as usize) {
            if let Some(u) = user {
                if u.is_active == 1 {
                    let token_str = bytes_to_string(&u.session_token);
                    if token_str.is_empty() {
                        println!("Skipping validation for user {} - empty token", u.user_id);
                        continue;
                    }
                    println!("Validating session user {}, token: '{}'", u.user_id, token_str);
                    println!("Token length: {}, bytes: {:?}", token_str.len(), u.session_token[0..16].to_vec());
                    let _ = self.c_extensions.validate_session(bytes_to_string(&u.session_token).as_str());
                }
            }
        }
    }
    //Read Only : Dont Change
    pub fn increase_day(&mut self) {
        println!("=== DEBUG: Starting increase_day ===");
        
        println!("=== DEBUG: About to sync_database ===");
        self.sync_database();
        println!("=== DEBUG: sync_database completed ===");
        
        println!("=== DEBUG: About to increment day counter ===");
        *(self._day_counter) += 1;
        println!("=== DEBUG: day counter incremented to {} ===", *self._day_counter);
        
        println!("=== DEBUG: About to validate_active_user_session ===");
        self.validate_active_user_session();
        println!("=== DEBUG: validate_active_user_session completed ===");
        
        println!("=== DEBUG: About to update_database_daily (Rust) ===");
        update_database_daily(&mut self.rust_db);
        println!("=== DEBUG: update_database_daily (Rust) completed ===");
        
        println!("=== DEBUG: About to call C increment_day ===");
        self.c_extensions.increment_day(&self.rust_db);
        println!("=== DEBUG: C increment_day completed ===");
    }

    pub fn print_both_databases(&self) {
        println!("---------------------------------C Backend Database State --------------------------------");
        self.c_extensions.print_database_full();
        println!("------------------------------------Rust Database State ----------------------------------");
        database_fix_full::print_database(&self.rust_db);
    }
}
fn create_small_test_data() -> Vec<DayData> {
    vec![
        DayData {
            day: 1,
            signups: Some(vec![
                UserEntry {
                    email: Some("test1@example.com".to_string()),
                    username: "user1".to_string(),
                    password: "pass1".to_string(),
                    id: Some(1),
                },
                UserEntry {
                    email: Some("test2@example.com".to_string()),
                    username: "user2".to_string(),
                    password: "pass2".to_string(),
                    id: Some(2),
                },
            ]),
            logins: None,
        },
    ]
}
fn main() {
    println!("=======Mixed Code Student Database System========");

    let mut db = EnhancedStudentDatabase::new();
    println!("mixed: database created");
    println!("About to test simple Vec creation...");
    // let test_vec: Vec<i32> = vec![1, 2, 3];
    // println!("Simple Vec created: {:?}", test_vec);

    // Initialize with static data
    // try moving into a box to move into heap
    // let days_data = create_small_test_data();
    let days_data = Box::new(generated_data::get_days_data());

    println!("Days created");
    // Process each day's activities
    for day_data in days_data.iter() {
        let mut local_session_tokens: Vec<String> = Vec::new();

        println!("============================[Info] Processing day {}===========================", day_data.day);

        if let Some(signups) = &day_data.signups {
            println!("=========[Info] Processing Signups============");
            for signup in signups {
                let username = signup.username.clone();
                let email = signup
                    .email
                    .clone()
                    .unwrap_or_else(|| "no-email@default.com".to_string());
                let password = signup.password.clone();

                match db.enqueue_user(
                    Box::leak(username.into_boxed_str()),
                    Box::leak(email.into_boxed_str()),
                    Box::leak(password.into_boxed_str()),
                ) {
                    Ok(_) => println!("[Signup] Queued user: {}", signup.username),
                    Err(e) => println!(
                        "[Signup Error] Failed to queue user {}: {}",
                        signup.username, e
                    ),
                }
            }
        }

        if let Some(logins) = &day_data.logins {
            println!("=========[Info] Processing Logins============");
            for login in logins {
                let password =  login.password.clone().chars().take(MAX_PASSWORD_LENGTH-1).collect::<String>();

                // Attempt user login
                match db.login_user(&login.username, &password) {
                    Ok(session_token) => {
                        println!("[Login] User {} logged in successfully", login.username);
                        local_session_tokens.push(session_token);
                    }
                    Err(e) => {
                        println!(
                            "[Login Error] Failed to login user {}: {}",
                            login.username, e
                        );
                    }
                }
            }
        }
        println!("========[Info] Performing end-of-day updates========");
        db.increase_day();

        println!(
            "=====[Info Day {}] Total Site traffic on Rust DB = {}======",
            day_data.day,
            local_session_tokens.len()
        );
    }

    println!("\n====================Congratulations! End of Simulation====================\n");

    db.print_both_databases();
    
    println!("\n==========================Did you really fix it ?======================================\n");
}
