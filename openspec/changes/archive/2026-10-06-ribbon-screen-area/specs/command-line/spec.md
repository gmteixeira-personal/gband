## ADDED Requirements

### Requirement: Server starting size
`gband server` SHALL accept the option `--size <cols>x<rows>`, where both are whole numbers above 0. The session the server creates on start SHALL start at that screen area, as the session-server capability's "Session on start" defines. When the option is absent, that area SHALL be 80 columns by 24 rows. When the value is not of that form, `gband` SHALL print one line to standard error naming the value and the form `COLSxROWS`, and exit with status 2, without creating or writing a log file.

#### Scenario: Server started at a size
- **WHEN** the user runs `gband server --size 100x30` and no client has attached
- **THEN** the session's first window, in a column of width 1/2, has a PTY of 48 columns by 28 rows

#### Scenario: Invalid size
- **WHEN** the user runs `gband server --size 0x24`
- **THEN** standard error names `0x24` and the form `COLSxROWS`
- **AND** the process exits with status 2
- **AND** no log file is created
