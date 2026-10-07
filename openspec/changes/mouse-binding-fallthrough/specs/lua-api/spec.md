## MODIFIED Requirements

### Requirement: API version rule
`gband.api_version` SHALL identify the documented API, whether the executable or a bundled Lua module provides a part of it. A build SHALL raise `gband.api_version` by one when it removes a documented function, table, field, primitive, provider function, module export, event or payload field, or changes the documented arguments, return values, errors or effect of one. A build that only adds to the documented API, or changes only what the documentation does not describe, SHALL keep it. The API documentation SHALL state this rule, and SHALL list what changed for each value of `gband.api_version` above 1.

Version 2 SHALL differ from version 1 in one documented effect: a function bound to a mouse name that returns `false` declines the press or wheel step, as the mouse capability's "Mouse names in key tables" defines, where in version 1 it replaced the default like any other binding.

#### Scenario: Version after this change
- **WHEN** `user/init.lua` reads `gband.api_version`
- **THEN** it reads `2`

#### Scenario: Rule documented
- **WHEN** a reader opens the "API and stability" section of `docs/plugins.md`
- **THEN** it states when `gband.api_version` changes, lists no change for version 1, and lists for version 2 that a function bound to a mouse name that returns `false` declines the press or wheel step
