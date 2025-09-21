#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

// Read Only - Do not modify
#define MAX_USERS 100
#define MAX_NAME_LEN 50
#define INACTIVITY_THRESHOLD 5
#define MAX_EMAIL_LEN 50
#define MAX_PASSWORD_LENGTH 100
#define SESSION_MAX_IDLE_TIME 1
#define MAX_SESSIONS 100
#define MAX_SESSION_TOKEN_LEN 32



typedef struct {
    char password[MAX_PASSWORD_LENGTH];
    char username[MAX_NAME_LEN];
    int user_id;
    char email[MAX_EMAIL_LEN];
    int inactivity_count;
    int is_active;
    char session_token[MAX_SESSION_TOKEN_LEN];
} UserStruct_t;

typedef struct {
    UserStruct_t *users[MAX_USERS];
    int count;
    int capacity;
} UserDatabase_t;

typedef struct {
    int user_id;
    char username[MAX_NAME_LEN];
    char session_token[MAX_SESSION_TOKEN_LEN];
    int session_idle_time;
    int is_active;
} SessionInfo_t;

typedef struct {
    SessionInfo_t *sessions[MAX_SESSIONS];
    int session_count;
    UserDatabase_t* db_ref;
} SessionManager_t;

// Global state for cross-language interaction
static SessionManager_t* global_session_manager = NULL;
static UserDatabase_t* global_db = NULL;
int *global_day_counter;

// Core database functions
UserDatabase_t* init_database(int* dc) {
    // UserDatabase_t* db = malloc(sizeof(UserDatabase_t));
    // db->count = 0;
    // db->capacity = MAX_USERS;
    // global_db = db;
    // return global_db;
    UserDatabase_t* db = malloc(sizeof(UserDatabase_t));
    if (db == NULL) {
        return NULL;  // Handle malloc failure
    }
    
    db->count = 0;
    db->capacity = MAX_USERS;
    
    for (int i = 0; i < MAX_USERS; i++) {
        db->users[i] = NULL;
    }
    
    global_db = db;
    global_day_counter = (int*)dc;
    return global_db;
}

void add_user(UserDatabase_t* db, UserStruct_t* user) {
    if (!db || !user) {
        printf("[C DEBUG] add_user: NULL parameters\n");
        return;
    }
    
    if (db->capacity != MAX_USERS) {
        printf("[C DEBUG] ERROR: Database corrupted! capacity=%d, expected=%d\n", 
               db->capacity, MAX_USERS);
        return;
    }
    
    if (db->count >= db->capacity) {
        printf("[C DEBUG] ERROR: Database full! count=%d\n", db->count);
        return;
    }
    #ifdef DEBUG_EN
    printf("[C-Code] Adding user: %s\n increasing count to %d\n", user->username, db->count + 1);
    #endif
    user->user_id = db->count + 1;
    db->users[db->count++] = user;
}

void free_user(UserStruct_t* user) {
    #ifdef DEBUG_EN
    printf("[C-Code] Freeing user: %s\n", user->username);
    #endif
    free(user);
}
void cleanup_database(UserDatabase_t* db) {
    for (int i = 0; i < db->count; i++) {
        free_user(db->users[i]);
    }
    free(db);
}

void print_database(UserDatabase_t *db) {
    // for(int i = 0; i < db->count; i++) {
    //     printf("User: %s, ID: %d, Email: %s, Inactivity: %d  Password = %s\n", db->users[i]->username, db->users[i]->user_id, db->users[i]->email, db->users[i]->inactivity_count, db->users[i]->password);
    // }
    // cleanup_database(db); why is this here
    if (!db) {
        printf("Database is NULL\n");
        return;
    }
    
    printf("Database has %d users (not printing details to avoid crash)\n", db->count);
    for(int i = 0; i < db->count; i++) {
        if (db->users[i] && is_valid_user_pointer(db->users[i])) {
            printf("User: %s, ID: %d, Email: %s, Inactivity: %d\n", 
                   db->users[i]->username, db->users[i]->user_id, 
                   db->users[i]->email, db->users[i]->inactivity_count);
        }
    }
}

void copy_string(char* dest, char* src, size_t n) {
    // for (size_t i = 0; i < n-1; i++)
    // {
    //     dest[i] = src[i];
    // }
    // dest[n-1] = '\0'; // Ensure null termination
    // if (!dest || !src || n == 0) {
    //     return;
    // }
    // size_t src_len = strlen(src);
    // size_t copy_len = (src_len < n- 1) ? src_len : n - 1;
    // memset(dest, 0, n);
    // for (size_t i = 0; i < copy_len; i++) {
    //     dest[i] = src[i];  
    // }
    // dest[copy_len] = '\0';  
    strncpy(dest, src, n - 1);
    dest[n - 1] = '\0';
}
int get_current_time() {
    return time(NULL);
}

UserStruct_t* create_user(char* username, char* email, int user_id, char* password) {
    printf("[C DEBUG] create_user called\n");
    printf("[C DEBUG] username ptr=%p\n", (void*)username);
    printf("[C DEBUG] email ptr=%p\n", (void*)email);
    printf("[C DEBUG] password ptr=%p\n", (void*)password);
    
    if (!username || !email || !password) {
        printf("[C DEBUG] ERROR: NULL parameter detected\n");
        return NULL;
    }
    
    printf("[C DEBUG] username='%s' (len=%zu)\n", username, strlen(username));
    printf("[C DEBUG] email='%s' (len=%zu)\n", email, strlen(email));
    printf("[C DEBUG] password='%s' (len=%zu)\n", password, strlen(password));

    UserStruct_t* user = malloc(sizeof(UserStruct_t));
    if (!user) return NULL;
    
    memset(user, 0, sizeof(UserStruct_t));
    
    printf("[C DEBUG] About to copy strings\n");
    copy_string(user->username, username, MAX_NAME_LEN);
    copy_string(user->email, email, MAX_EMAIL_LEN);
    copy_string(user->password, password, MAX_PASSWORD_LENGTH);
    
    printf("[C DEBUG] After copying - username='%s'\n", user->username);
    printf("[C DEBUG] After copying - email='%s'\n", user->email);
    printf("[C DEBUG] After copying - password='%s'\n", user->password);
    
    user->user_id = user_id;
    user->inactivity_count = 0;
    user->is_active = 1;
    
    return user;
}
void update_day_counter(int *day_counter) {
    global_day_counter = day_counter;
}

UserStruct_t* find_user_by_id(UserDatabase_t* db, int user_id) {
    if (!db) {
        return NULL;
    }
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] && db->users[i]->user_id == user_id) {
            return db->users[i];
        }
    }
    return NULL;
}
int init_session_manager() {
    if (global_session_manager != NULL) {
        return 0;
    }
    
    global_session_manager = malloc(sizeof(SessionManager_t));
    if (!global_session_manager) {
        return -1;
    }
    
    global_session_manager->session_count = 0;
    global_session_manager->db_ref = global_db;
    // memset(global_session_manager->sessions, 0, sizeof(SessionInfo_t) * MAX_SESSIONS);
    for (int i = 0; i < MAX_SESSIONS; i++) {
        global_session_manager->sessions[i] = NULL;
    }
    #ifdef DEBUG_EN
    printf("[C-Code] Session manager initialized\n");
    #endif
    return 0;
}

void generate_token(char *token,char *name, int timestamp){
    char temp[MAX_SESSION_TOKEN_LEN];
    snprintf(temp, sizeof(temp), "session_%s_%d", name, timestamp);
    copy_string(token, temp, MAX_SESSION_TOKEN_LEN);

  
}

char* create_user_session(UserStruct_t *user) {
    if (!global_session_manager) {
        if(init_session_manager()){
            return NULL;
        }
    }

    if (!user) {
        return NULL;
    }
    if (strlen(user->username) == 0) {
        printf("[C DEBUG] ERROR: User has empty username!\n");
        return NULL;
    }

    if (global_session_manager->session_count >= MAX_SESSIONS) {
        #ifdef DEBUG_EN
        printf("[C-Code] Too many active sessions\n");
        #endif
        exit(1);
    }
    
    char* token = malloc(MAX_SESSION_TOKEN_LEN);
    generate_token(token, user->username, get_current_time());


    SessionInfo_t* session = malloc(sizeof(SessionInfo_t));
    session->user_id = user->user_id;
    copy_string(session->username, user->username, MAX_NAME_LEN);
    copy_string(session->session_token, token, MAX_SESSION_TOKEN_LEN);
    session->is_active = 1;
    session->session_idle_time = 0;
    global_session_manager->sessions[global_session_manager->session_count] = session;
    global_session_manager->session_count++;

    #ifdef DEBUG_EN
    printf("Created session for user %d: %s\n", user_id, token);
    #endif
    return token;
}

// Memory management and optimization functions
int get_non_null_ref_count(UserDatabase_t* db) {
    int count = 0;
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] != NULL) {
            count++;
        }
    }
    return count;
}

//Hint : Interesting function
UserStruct_t** get_user_reference_for_debugging(UserDatabase_t* db) {
    // UserStruct_t **user;
    int non_null = get_non_null_ref_count(db);

    #ifdef DEBUG_EN
    printf("[C-Code] Scanning database for non-null users... among %d users\n", db->count);
    #endif
    if (non_null == 0) {
        return NULL;
    }
    // *user = malloc(sizeof(non_null * sizeof(UserStruct_t*)));
    UserStruct_t **user_array = malloc(non_null * sizeof(UserStruct_t*));
    if (!user_array) {
        return NULL;
    }

    int index = 0;
    for(int i = 0; i < db->count && index < non_null; i ++) {
        UserStruct_t* useri = db->users[i];
        if(useri != NULL){
            #ifdef DEBUG_EN
            printf("[C-Code] Adding user reference for %s\n", useri->username);
            #endif
            user_array[index] = useri;  // FIX: Forward indexing
            index++;
        }
    }
    return user_array;
}



void clone_user(UserStruct_t* src, UserStruct_t* dest) {
    copy_string(dest->username, src->username, MAX_NAME_LEN);
    copy_string(dest->email, src->email, MAX_EMAIL_LEN);
    copy_string(dest->password, src->password,MAX_PASSWORD_LENGTH);
    dest->inactivity_count = src->inactivity_count;
    copy_string(dest->session_token, src->session_token, MAX_SESSION_TOKEN_LEN);
    dest->is_active = src->is_active;
}

//Hint : Interesting function
void memory_pressure_cleanup(UserDatabase_t* db) {
    // #ifdef DEBUG_EN
    // printf("[C-Code] System under memory pressure - performing selective cleanup\n");
    // #endif
    // // shift users together and compact the array
    // int cnt = get_non_null_ref_count(db);
    // for (int i = 0; i < db->count; i++) {
    //     if (db->users[i] == NULL) {
    //         // Shift non-null users down
    //         for (int j = i + 1; j < db->count; j++) {
    //             if (db->users[j] != NULL) {
    //                 clone_user(db->users[j], db->users[i]);
    //                 free_user(db->users[j]);
    //                 break;
    //             }
    //         }
    //     }
    // }
    // db->count = cnt;
    // #ifdef DEBUG_EN
    // printf("Memory pressure cleanup completed\n");
    // #endif
    if (!db) {
        return;
    }
    int cnt = get_non_null_ref_count(db);
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] == NULL) {
            for (int j = i + 1; j < db->count; j++) {
                if (db->users[j] != NULL) {
                    // allocate memory before cloning, make as truct
                    db->users[i] = malloc(sizeof(UserStruct_t));
                    if (db->users[i] != NULL) {  
                        clone_user(db->users[j], db->users[i]);
                        free_user(db->users[j]);
                        db->users[j] = NULL;  
                    }
                    break;
                }
            }
        }
    }
    db->count = cnt;
    #ifdef DEBUG_EN
    printf("Memory pressure cleanup completed\n");
    #endif
}




SessionInfo_t* find_session_by_token(SessionManager_t* sm, char* token) {
    for (int i = 0; i < sm->session_count; i++) {
        if (sm->sessions[i]->is_active && strcmp(sm->sessions[i]->session_token, token) == 0) {
            return sm->sessions[i];
        }
    }
    return NULL;
}


int validate_user_session(char* token) {
    
    if (!global_session_manager || !token) {
        return 0;
    }

    SessionInfo_t *session = find_session_by_token(global_session_manager, token);
    if (!session) {
        return 0;
    }
    if (session->session_idle_time > SESSION_MAX_IDLE_TIME) {

        session->is_active = 0;
        // free(session); // dont free let deact_usersers handle MAYBENOT
        return 1;
    }
    else session->session_idle_time += 1;
    return 0;
}

void merge_duplicate_handles(UserDatabase_t *db){
    if (!db) {
        return;
    }
    for(int i = (db->count-1); i >= 0; i--){
        if (!db->users[i]) continue;
        for(int j = 0; j < i; j++){
            if (!db->users[j]) continue;
            if(strcmp(db->users[i]->username, db->users[j]->username) == 0 && strcmp(db->users[i]->email, db->users[j]->email) == 0 && strcmp(db->users[i]->password, db->users[j]->password) == 0){
                #ifdef DEBUG_EN
                printf("[C-Code] Merging duplicate user handles for %s\n", db->users[i]->username);
                #endif
                free_user(db->users[j]);
                db->users[j] = NULL;
            }
        }
    }
}
//#REMOVE debugging function
int is_valid_user_pointer(UserStruct_t* user) {
    if (!user) return 0;
    
    // Basic sanity checks
    if (user->user_id <= 0 || user->user_id > 10000) return 0;
    if (user->inactivity_count < 0 || user->inactivity_count > 1000) return 0;
    if (user->is_active != 0 && user->is_active != 1) return 0;
    
    return 1;
}
void update_database_daily(UserDatabase_t* db) {
    // if (!db) {
    //     printf("database is null");
    // }
    // if (!db || !global_day_counter) return;
    // for (int i = 0; i < db->count; i++) {
    //     if (!db->users[i]) continue;
    //     if (!is_valid_user_pointer(db->users[i])) {
    //         printf("[C ERROR] Invalid user pointer at index %d\n", i);
    //         db->users[i] = NULL;
    //         continue;
    //     }
    //     printf("ok continuing");
    //     if (!db->users[i]->is_active && db->users[i]->inactivity_count > INACTIVITY_THRESHOLD) {
    //         #ifdef DEBUG_EN
    //             printf("[C-Code] Removing user[%d] %s due to inactivity for %d days\n", db->users[i]->user_id, db->users[i]->username, db->users[i]->inactivity_count);
    //         #endif
    //         free_user(db->users[i]);
    //         db->users[i] =  NULL;
    //     } else {
    //         if(validate_user_session(db->users[i]->session_token)) db->users[i]->is_active = 0;
    //         #ifdef DEBUG_EN
    //             printf("[C-Code] %d is max allowed threshold Incrementing inactivity for user[%d] %s to %d days\n", INACTIVITY_THRESHOLD, db->users[i]->user_id, db->users[i]->username, db->users[i]->inactivity_count + 1);
    //           #endif
    //         db->users[i]->inactivity_count++;
    //     }
    // }
        if (!db) {
        printf("[C ERROR] update_database_daily: NULL database\n");
        return;
    }
    
    printf("[C DEBUG] Starting update_database_daily with %d users\n", db->count);
    
    for (int i = 0; i < db->count; i++) {
        printf("[C DEBUG] Processing user %d...\n", i);
        
        if (!db->users[i]) {
            printf("[C DEBUG] User %d is NULL\n", i);
            continue;
        }
        
        if (!is_valid_user_pointer(db->users[i])) {
            printf("[C ERROR] User %d failed validation\n", i);
            db->users[i] = NULL;
            continue;
        }
        
        printf("[C DEBUG] User %d passed validation, checking activity...\n", i);
        
        // Add extra safety checks before accessing fields
        int is_active = db->users[i]->is_active;
        int inactivity = db->users[i]->inactivity_count;
        
        printf("[C DEBUG] User %d: is_active=%d, inactivity=%d\n", i, is_active, inactivity);
        
        if (!is_active && inactivity > INACTIVITY_THRESHOLD) {
            printf("[C DEBUG] Removing inactive user %d\n", i);
            free_user(db->users[i]);
            db->users[i] = NULL;
        } else {
            printf("[C DEBUG] Updating user %d\n", i);
            db->users[i]->inactivity_count++;
        }
    }
    printf("[C DEBUG] update_database_daily completed\n");

    if(*global_day_counter % 4 == 0){
        merge_duplicate_handles(db);
    }

    if (*global_day_counter % 8 == 0){
        memory_pressure_cleanup(db);
    }
}

UserStruct_t* find_user_by_username(UserDatabase_t* db, char* user_name) {
    for (int i = 0; i < db->count; i++) {
        if (!db->users[i]) continue;
        if (strcmp(db->users[i]->username, user_name) == 0) {
            return db->users[i];
        }
    }
    return NULL;
}


char* user_login(UserDatabase_t* db, char* user_name) {
    UserStruct_t* user = find_user_by_username(db, user_name);
    if (!db || !user_name) {
        return NULL;
    }
    #ifdef DEBUG_EN
    printf("[C-Code] User[%d] %s logged in after %d days\n", user->user_id, user->username, user->inactivity_count);
    #endif
    user->inactivity_count = 0;
    char *token = create_user_session(user);
    copy_string(user->session_token, token, MAX_SESSION_TOKEN_LEN);
    user->is_active = 1;
    return token;
}

char* get_password(UserDatabase_t* db, char* username) {
    if (!db || !username) return NULL;

    UserStruct_t* user = find_user_by_username(db, username);
    if (!user) return NULL;
    #ifdef DEBUG_EN
        printf("Password  Request for User[%d] %s is %s\n", user->user_id, user->username, user->password);
    #endif
    return user->password;
}




UserStruct_t* find_user_by_session_token(UserDatabase_t* db, char* session_token) {
    for (int i = 0; i < db->count; i++) {
        if (db->users[i] != NULL && strcmp(db->users[i]->session_token, session_token) == 0) {
            return db->users[i];
        }
    }
    return NULL;
}


void deactivate_users(UserDatabase_t* rust_db) {
    if (!global_session_manager || !rust_db) {
        printf("[C DEBUG] deactivate_users: NULL parameters\n");
        return;
    }
    for (int i = 0; i < global_session_manager->session_count; i++) {
        if (!global_session_manager->sessions[i]) continue;
        if (global_session_manager->sessions[i]->session_idle_time > SESSION_MAX_IDLE_TIME) {
            global_session_manager->sessions[i]->is_active = 0;
        }
        
        UserStruct_t *user = find_user_by_session_token(global_db, global_session_manager->sessions[i]->session_token);
        if (user) {
            user->is_active = 0;
        }

        
        user = find_user_by_session_token(rust_db, global_session_manager->sessions[i]->session_token);
        if (user) {
            user->is_active = 0;
        }

        
        free(global_session_manager->sessions[i]);
        global_session_manager->sessions[i] = NULL;
    }
}

