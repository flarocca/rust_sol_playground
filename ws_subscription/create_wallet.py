import os
import json
from base58 import b58decode
import getpass

# Prompt for the file name (without extension)
file_name = input("Enter the file name (without extension): ")

# Build the file path in /Users/<user>/.config/solana/
home_dir = os.path.expanduser("~")
config_dir = os.path.join(home_dir, ".config", "solana")
os.makedirs(config_dir, exist_ok=True)  # Create directory if it doesn't exist
file_path = os.path.join(config_dir, f"{file_name}.json")

# Securely prompt for the Base58 private key
base58_key = getpass.getpass(prompt="Enter your Base58 private key: ")

# Decode the Base58 key
decoded = b58decode(base58_key)

# Convert the result to a list of numbers
decoded_array = list(decoded)

# Save the list to the specified JSON file
with open(file_path, 'w') as f:
    json.dump(decoded_array, f)

print(f"Decoded key saved to {file_path}") 
