---
name: migration-prefix
description: Generate timestamp prefix for database migration files in YYYYMMDDHHMMSS format. Use when creating new SQL migration files or when the user needs to generate a migration file timestamp prefix.
---

# Migration File Prefix Generator

## Purpose

Generate timestamp prefixes for database migration files following the project's naming convention: `YYYYMMDDHHMMSS_description.sql`

## Quick Usage

When you need to create a new migration file, use this command to generate the timestamp prefix:

```bash
date +"%Y%m%d%H%M%S"
```

This will output a timestamp like: `20260512143025`

## Complete Migration File Name

After getting the timestamp, construct the full filename:

```
{timestamp}_{description}.sql
```

### Examples

- `20260512143025_create_users_table.sql`
- `20260512143026_add_email_to_users.sql`
- `20260512143027_update_user_status_constraint.sql`

## Naming Rules

1. **Timestamp format**: `YYYYMMDDHHMMSS` (14 digits)
   - Generated using: `date +"%Y%m%d%H%M%S"`
2. **Description format**:
   - Use lowercase English words
   - Separate words with underscores
   - Be descriptive but concise
   - Examples: `create_novels`, `add_thinking_content`, `fix_column_order`

3. **File extension**: Always `.sql`

## Workflow

1. Run `date +"%Y%m%d%H%M%S"` to get the timestamp
2. Decide on a descriptive name for the migration
3. Combine them: `{timestamp}_{description}.sql`
4. Create the file in `src-tauri/migrations/`

## Important Notes

- Always use the current timestamp when creating a new migration
- Ensure the description clearly indicates what the migration does
- Follow existing migration file patterns in the project
- Timestamps ensure migrations are applied in chronological order
