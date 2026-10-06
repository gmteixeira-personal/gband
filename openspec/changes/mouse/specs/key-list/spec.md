## ADDED Requirements

### Requirement: Mouse bindings in the key list
The key list SHALL hold no line for a binding whose key is a mouse name, as the configuration capability defines mouse names. The order of the other lines SHALL be unchanged.

#### Scenario: Default key list
- **WHEN** the default configuration is in use and the user opens the key list
- **THEN** no line shows `leftmouse`, `rightmouse`, `middlemouse` or a wheel name
