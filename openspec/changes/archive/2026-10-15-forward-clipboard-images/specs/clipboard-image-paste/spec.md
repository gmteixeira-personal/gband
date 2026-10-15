## ADDED Requirements

### Requirement: The paste image action
The client SHALL provide the built-in client action `paste_image`, as the actions capability lists it. Dispatching it SHALL, when a window is focused, read an image from the clipboard of the host the client runs on and forward it to the server for the focused window, as "Reading the clipboard" and the wire-protocol capability define. When no window is focused, it SHALL send nothing.

#### Scenario: Paste an image into the focused window
- **WHEN** a window is focused, the clipboard holds a PNG image, and the client dispatches `paste_image`
- **THEN** the client sends a paste image message naming the focused window, the MIME type `image/png` and the image's bytes

#### Scenario: Nothing focused
- **WHEN** no window is focused and the client dispatches `paste_image`
- **THEN** the client sends nothing

### Requirement: Reading the clipboard
The client SHALL read the clipboard with the reader for its host: `wl-paste` when a Wayland display is set, else `xclip` when an X11 display is set, else `pbpaste` on macOS. It SHALL choose the `image/png` MIME type when the clipboard offers it, and else the first image MIME type the clipboard offers. It SHALL read the bytes of the chosen type and forward them with that type. When the reader is not installed, when the clipboard offers no image type, or when the reader fails, the client SHALL add one error to its error list, as the error-list capability defines, and SHALL send nothing.

#### Scenario: Prefer PNG
- **WHEN** the clipboard offers `image/png` and `image/bmp` and the client dispatches `paste_image`
- **THEN** the client forwards the `image/png` bytes

#### Scenario: Any image when no PNG
- **WHEN** the clipboard offers only `image/jpeg` and the client dispatches `paste_image`
- **THEN** the client forwards the `image/jpeg` bytes with the MIME type `image/jpeg`

#### Scenario: Reader not installed
- **WHEN** no clipboard reader is installed and the client dispatches `paste_image`
- **THEN** the client adds one error naming the missing reader to its error list and sends nothing

#### Scenario: No image on the clipboard
- **WHEN** the clipboard offers only text and the client dispatches `paste_image`
- **THEN** the client adds one error reporting no image to its error list and sends nothing

### Requirement: Writing the image and pasting its path
On a paste image message for a window of the client's session, the server SHALL write the bytes to a new file in a `gband-images` subdirectory of the base directory, and SHALL inject that file's absolute path into the window's program as a bracketed paste, as the session-server capability defines a paste. The base directory SHALL be the `clipboard_image_dir` option when it is set, and else the operating system's temporary directory. The file's name SHALL end in an extension for the MIME type: `.png` for `image/png`, `.jpg` for `image/jpeg`, and `.bin` otherwise. The server SHALL create the subdirectory when it is absent. The file SHALL be written on the host the server runs on, so `gband -r <host>` writes it on the remote host, where the window's program reads it.

#### Scenario: The path is pasted
- **WHEN** the server receives a paste image message naming window 1, the MIME type `image/png` and some bytes
- **THEN** the server writes the bytes to a `.png` file under `gband-images` in the temporary directory
- **AND** window 1's program receives the file's absolute path as a bracketed paste

#### Scenario: Configured directory
- **WHEN** `clipboard_image_dir` is set to `/tmp/shots` and the server receives a paste image message naming the MIME type `image/jpeg`
- **THEN** the server writes a `.jpg` file under `/tmp/shots/gband-images`

### Requirement: Pruning old images
Before writing a new image, the server SHALL delete every file in the `gband-images` subdirectory whose modification time is more than 1 hour old, and SHALL keep the file it is about to write. A failure to delete a file SHALL NOT stop the write.

#### Scenario: Old file pruned
- **WHEN** the `gband-images` directory holds a file modified 2 hours ago and the server writes a new image there
- **THEN** the old file is deleted and the new file remains

#### Scenario: Recent file kept
- **WHEN** the `gband-images` directory holds a file modified 10 minutes ago and the server writes a new image there
- **THEN** both files remain

### Requirement: The clipboard image directory option
The server option `clipboard_image_dir` SHALL name the base directory under which the server writes forwarded images, as a string, or SHALL be unset. Unset SHALL mean the operating system's temporary directory. Its default SHALL be unset. It SHALL appear in `gband.opt` and in `gband.opt` listings like the other options.

#### Scenario: Default is unset
- **WHEN** `gband.opt.clipboard_image_dir` is read with no configuration setting it
- **THEN** it is unset, so the server writes under the operating system's temporary directory

#### Scenario: Set to a directory
- **WHEN** a configuration sets `gband.opt.clipboard_image_dir` to `/tmp/shots`
- **THEN** the server writes forwarded images under `/tmp/shots/gband-images`
