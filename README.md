# Kragle: REPO Schema Dump and Validator


Kragle provides utilities to **convert a folder (directory tree) into a YAML representation** (optionally with file content compression) and to **recreate the original folder structure from such a YAML file**.
It can be used for archiving, transferring, or validating folder contents in a platform-agnostic way.

<div align="center">

![Kragle Logo](https://github.com/jxtopher/repo-schema-validator/blob/main/icon/kragle-128x128.png?raw=true)

</div>

## Features

- Converts a folder and its subfolders/files into a structured YAML.
- Optionally compresses file contents using lzma and encodes them with base85
- Stores file metadata: original name, size, MD5 hash, and content (compressed/uncompressed).
- Reconstructs the original folder and file structure from the YAML, decompressing and verifying MD5 hashes automatically.
- Supports both text and compressed (binary) file contents.

## Usage

### 1. Export a Folder to YAML

```
```

### 2. Recreate a Folder from YAML


```
```

### 3. Command-line Example

You may use the script directly by editing the `__main__` section:

```
```

## YAML Structure

Each directory is represented as:

```YAML
name: directory_name
type: directory
children:
  - ...
```

Each file is represented as:

```YAML
name: filename.txt
type: file
original_size: 123
original_md5: md5hash...
is_compressed: true
content: "...  # base85-encoded lzma or raw text"
```

## File Verification

When reconstructing, the script computes the MD5 hash of each written file and compares it to the hash stored in the YAML. Any mismatch will be reported in the output.
