import random
import yaml
import sys
import time
import os


usernames = [
    "AliceAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "Bob_the_destroyer",
    "CharlieLOSTTHEfactory",
    "../../../../etc/passwd",
    "Eve<script>alert('pwned')</script>",
    "Mallory",
    "Trudy_with_a_very_very_very_long",
    "Oscar",
    "FinalBufferOverflow",
    "NullUserInjected",
    "Underoverriveroverflow",
    "StackSmasher9000",
    "INT_MIN_User",
    "Buffer_The_Magic_Dragon",
    "root:toor",
]

emails = [
    "alice@nus.edu.sg",
    "bob@over.flow",
    "charlie@longdomainnamethatshouldnotexistbecauseitbreaks.memory.safety.edu.sg",
    "root@localhost",
    "eve@xss.attack",
    "mallory@evil.corp",
    "trudy@overflowy.com",
    "oscar@@doubleatsign.com",
    "segfault@0xdeadbeef",
    "null@pointer.exception",
    "emoji@.com",
    "stack@smash.me",
    "minint@underflow.net",
    "buffer@dragon.fire",
    "admin@rootkit.org",
]

passwords = [
    "aliceinthewonderland",
    "hunter2",
    "passwordpasswordpasswordpasswordpasswordpasswordpassword",
    "toomanybytes_to_fit_in_static_array_buffer_but_we_try_anyway!!!",
    "killedthedbnowiamhappy",
    "stacksmashstacksmashstacksmashstacksmash",
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "",
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "letmein123456789012345678901234567890",
    "correcthorsebatterystapleBUToverflowed",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "minint_overflow",
    "eavesdroppingagain",
    "password_is_too_damn_long_" + "X" * 200,
]

yaml_entries = []

#  day n = 5-10 randome users
# day n+1 = n/2 of them randomly in login set


random.seed(time.time())

def create_signup_set():
    count = random.randint(6, 11)
    signup_set = []
    for i in range(count):
        user_name = str(random.randint(1, 10000)) + random.choice(usernames)
        user_password = random.choice(passwords)
        user_email = random.choice(emails)
        user = {
            "id": i+1,
            "username": user_name,
            "email": user_email,
            "password": user_password,
        }
        signup_set.append(user)
    return signup_set

def get_login_set_for_day(user_set_prev_day):
    count = len(user_set_prev_day) // 2
    user_set = user_set_prev_day.copy()
    login_set = []
    for i in range(count):
        random_user = random.choice(user_set)
        user = {
            "id": i+1,
            "username": random_user["username"],
            "password": random_user["password"],
        }
        user_set.remove(random_user)
        login_set.append(user)
    
    return login_set


def simulate_day(n):
    signup_set = []
    yaml_entries = []
    for day in range(n):
        login_set = get_login_set_for_day(signup_set)
        signup_set = create_signup_set()
        day_entry = {
            "day": day + 1,
            "signups": signup_set,
            "logins": login_set
        }
        yaml_entries.append(day_entry)
    return yaml_entries


def main():
    n = int(sys.argv[1])
    yaml_data = simulate_day(n)
    with open("db.yaml", "w", encoding="utf-8") as f:
        f.write(yaml.dump(yaml_data))
    os.system("python3 generate_rust_data.py db.yaml src/generated_data.rs")

if __name__ == "__main__":
    main()