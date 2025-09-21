#!/usr/bin/env python3
import yaml
import sys

def escape_rust_string(s):
    return s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t')

def generate_rust_code(yaml_file, output_file):

    with open(yaml_file, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    
    rust_code = '''// Auto-generated from YAML data - DO NOT EDIT MANUALLY
use crate::{UserEntry, DayData};

pub fn get_days_data() -> Vec<DayData> {
    vec![
'''
    
    for day_entry in data:
        rust_code += f'        DayData {{\n'
        rust_code += f'            day: {day_entry["day"]},\n'
        
        # Handle signups
        if 'signups' in day_entry and day_entry['signups']:
            rust_code += '            signups: Some(vec![\n'
            for signup in day_entry['signups']:
                email = escape_rust_string(signup.get('email', ''))
                username = escape_rust_string(signup['username'])
                password = escape_rust_string(signup['password'])
                id_val = signup.get('id', 'None')
                id_str = f'Some({id_val})' if id_val != 'None' else 'None'
                
                rust_code += f'                UserEntry {{\n'
                rust_code += f'                    email: Some("{email}".to_string()),\n'
                rust_code += f'                    username: "{username}".to_string(),\n'
                rust_code += f'                    password: "{password}".to_string(),\n'
                rust_code += f'                    id: {id_str},\n'
                rust_code += f'                }},\n'
            rust_code += '            ]),\n'
        else:
            rust_code += '            signups: None,\n'
        
        # Handle logins
        if 'logins' in day_entry and day_entry['logins']:
            rust_code += '            logins: Some(vec![\n'
            for login in day_entry['logins']:
                username = escape_rust_string(login['username'])
                password = escape_rust_string(login['password'])
                id_val = login.get('id', 'None')
                id_str = f'Some({id_val})' if id_val != 'None' else 'None'
                
                rust_code += f'                UserEntry {{\n'
                rust_code += f'                    email: None,\n'  # Logins don't have email in your YAML
                rust_code += f'                    username: "{username}".to_string(),\n'
                rust_code += f'                    password: "{password}".to_string(),\n'
                rust_code += f'                    id: {id_str},\n'
                rust_code += f'                }},\n'
            rust_code += '            ]),\n'
        else:
            rust_code += '            logins: None,\n'
        
        rust_code += '        },\n'
    
    rust_code += '    ]\n'
    rust_code += '}\n'
    
    # Write to output file
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(rust_code)
    
    print(f"Generated Rust code in {output_file}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python3 generate_rust_data_lazy.py <input.yaml> <output.rs>")
        sys.exit(1)
    
    yaml_file = sys.argv[1]
    output_file = sys.argv[2]
    
    generate_rust_code(yaml_file, output_file)
