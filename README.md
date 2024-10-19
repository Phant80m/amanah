# amanah
أمانة

A command-line password manager written in Rust. "Amanah" (أمانة) means trust, and this tool helps you securely manage your passwords.

## Features

- Store and retrieve encrypted passwords.
- Search passwords using fuzzy matching (Levenshtein distance).
- Simple interface to add, list, retrieve, and delete password entries.
- Encrypted password storage with `MiniCocoon`.

## Installation

To install **amanah**, use the following command:

```bash
cargo install --git https://github.com/Phant80m/amanah
```

## Usage

**amanah** uses subcommands for different operations: `add`, `list`, `get`, and `remove`.

### Add a New Password

To add a new password:

```bash
amanah add <label> <username> <password>
```

- `<label>`: A unique identifier for the entry (e.g., "GitHub").
- `<username>`: The associated username for the password.
- `<password>`: The password itself (will be stored in encrypted format).

Example:

```bash
amanah add GitHub myusername mypassword123
```

### List All Passwords

To list all saved passwords:

```bash
amanah list
```

This will display all password entries (with decrypted passwords).

Example output:

```plaintext
Label: GitHub
Username: myusername
Password: mypassword123
```

### Retrieve a Password

To retrieve a password by its label or a fuzzy search query:

```bash
amanah get <label>
```

If an exact match for the label is not found, **amanah** will suggest the closest matching entry.

Example:

```bash
amanah get Githu
```

Possible output:

```plaintext
No password for Githu, did you mean: GitHub?
Label: GitHub
Username: myusername
Password: mypassword123
```

### Remove a Password

To remove a password by its label:

```bash
amanah remove <label>
```

This will prompt for confirmation before deleting the entry.

Example:

```bash
amanah remove GitHub
```

Output:

```plaintext
Type 'Delete: GitHub.' to confirm deletion.
```

Once confirmed, the entry will be permanently deleted.

## Security

- Passwords are encrypted using the `MiniCocoon` library.
- Encrypted data is stored in an SQLite database (`passwords.db`) located in the user’s config directory.

## Development

To contribute or modify this project, clone the repository:

```bash
git clone https://github.com/Phant80m/amanah
cd amanah
```

Build the project locally:

```bash
cargo build
```

Run the project:

```bash
cargo run -- <subcommand> <args>
```

Example:

```bash
cargo run -- add GitHub myusername mypassword123
```

## License

This project is licensed under the MIT License.
