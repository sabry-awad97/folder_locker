<div align="center">

# Locker

<svg width="200" height="200" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
  <rect x="40" y="80" width="120" height="100" fill="#4A4A4A"/>
  <rect x="60" y="20" width="80" height="60" rx="10" fill="#4A4A4A"/>
  <circle cx="100" cy="120" r="20" fill="#FFFFFF"/>
  <rect x="95" y="115" width="10" height="25" fill="#4A4A4A"/>
</svg>

**Folder Security Solution**

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Documentation](#documentation) • [Contributing](#contributing) • [License](#license)

</div>

---

## Introduction

Locker is a powerful command-line tool designed to provide enterprise-grade security for your sensitive folders. Locker transforms ordinary directories into impenetrable vaults, offering unparalleled data protection for individuals and organizations.

## Features

- **Military-grade Encryption**: Utilizes AES-256 encryption through bcrypt to ensure maximum data security.
- **Stealth Mode**: Implements advanced techniques to make protected folders undetectable.
- **Cross-platform Compatibility**: Seamlessly operates on Windows systems (with potential for expansion to other platforms).
- **Intuitive CLI**: User-friendly interface designed for both novices and experienced users.
- **Secure Password Management**: Implements robust password hashing and verification.
- **File Attribute Manipulation**: Utilizes Windows-specific commands to enhance folder security.
- **Progress Indication**: Provides visual feedback during locking and unlocking operations.

## Installation

### Using Cargo (Recommended)

For users with Rust and Cargo installed:

```bash
cargo install locker
```

### Manual Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/sabry-awad97/folder_locker.git
   cd locker
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. The binary will be available in `target/release/locker`.

## Usage

To secure a folder:

```bash
locker lock <FOLDER_PATH>
```

To unlock a folder:

```bash
locker unlock <FOLDER_PATH>
```

## Documentation

For detailed usage instructions and API documentation, run:

```bash
cargo doc --open
```

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more details.

## License

Locker is released under the MIT License. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
© 2024 Locker Security Solutions. All rights reserved.
</div>
