## MODIFIED Requirements

### Requirement: Terminal palette
The client SHALL keep one terminal palette: a set of up to 18 colors, each optional, with the fields `fg`, `bg` and the 16 color names of the highlights capability, from `black` to `bright_white`. Each color SHALL be a `#` followed by six hexadecimal digits of either case. The palette SHALL be empty when loading starts, before the start theme loads.

`gband.palette.set(spec)` SHALL replace the whole palette with `spec`, a table of those fields. `gband.palette.set({})` SHALL empty it. A `spec` that is not a table, a field not listed, or a value that is not such a hex color SHALL be an error at the line of the call naming the field, and SHALL leave the palette unchanged. `gband.palette.get()` SHALL return a new table of the palette's fields, with hex colors in lowercase. Changing a returned table SHALL NOT change the palette. Both functions SHALL be callable while the configuration loads and in any callback. After the configuration has loaded, a change of the palette SHALL redraw the client.

When the client draws its terminal, after every other step of drawing, it SHALL map each cell's foreground and background:
- the terminal's default foreground SHALL become `fg`;
- the terminal's default background SHALL become `bg`;
- palette index `n`, from 0 to 15, SHALL become the field of the `n`-th color name, `black` being index 0.

Each mapping SHALL apply only when the palette sets that field. It SHALL cover every cell of the terminal: windows' program output, borders, the empty ribbon, bars, plugin windows and the banner. The one exception SHALL be the foreground of a cell whose text begins with U+10EEEE, a kitty graphics placeholder, which SHALL be drawn unchanged, as the window-graphics capability's "Placement by Unicode placeholder" defines. A mapped color SHALL be drawn as a hex color, as the highlights capability defines, so a client without 24-bit color draws the nearest palette index from 16 to 255. Palette indexes 16 to 255 and hex colors SHALL be drawn unchanged. The palette SHALL change nothing the server stores or sends, and nothing a program in a window reads.

#### Scenario: Program color mapped
- **WHEN** the active colorscheme is `gruvbox`, the client supports 24-bit color, and a window runs `printf '\033[31mx\033[0m'`
- **THEN** the client draws `x` with the foreground `#cc241d` and the background `#282828`

#### Scenario: Empty ribbon takes the background
- **WHEN** the active colorscheme is `gruvbox` and part of the ribbon area holds no window
- **THEN** the client draws those cells with the background `#282828`

#### Scenario: Mapped color without 24-bit color
- **WHEN** the active colorscheme is `gruvbox`, the client does not support 24-bit color, and a window prints text in ANSI red
- **THEN** the client draws the text with palette index 160

#### Scenario: 256-color output unchanged
- **WHEN** the active colorscheme is `gruvbox` and a window runs `printf '\033[38;5;208mx'`
- **THEN** the client draws `x` with palette index 208

#### Scenario: Terminal theme leaves colors alone
- **WHEN** the active colorscheme is `terminal` and a window prints text in ANSI red on the default background
- **THEN** the client draws the text with palette index 1 and the terminal's default background

#### Scenario: Invalid palette field
- **WHEN** line 3 of `user/colors/dusk.lua` calls `gband.palette.set({ purple = "#800080" })`
- **THEN** an error at that line names `purple` and the palette is unchanged

#### Scenario: Invalid palette color
- **WHEN** line 2 of `user/init.lua` calls `gband.palette.set({ bg = 236 })`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `bg`

#### Scenario: Palette set by the init file
- **WHEN** `user/init.lua` calls `gband.palette.set({ bg = "#101010" })` and a window prints text with the default colors
- **THEN** the client draws the text on the background `#101010`, with the terminal's default foreground

#### Scenario: Placeholder foreground unchanged
- **WHEN** the active colorscheme is `gruvbox` and a window holds a cell whose text is U+10EEEE with a foreground of the 256-colour index 3
- **THEN** the client draws that cell with palette index 3 as its foreground
