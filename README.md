# dedent-paste

Paste indented text without the extra common indentation.

`dedent-paste` is a small macOS helper for Karabiner-Elements. Press `Option+V`, and it will:

1. Read the current clipboard as plain text.
2. Remove the common leading spaces or tabs from non-blank lines.
3. Paste the cleaned text with `Command+V`.

## Quick install

Requirements: macOS, Karabiner-Elements, `curl`, and Python 3.

```sh
curl -fsSL https://raw.githubusercontent.com/doggy8088/dedent-paste/main/install.sh | bash
```

After installation, press:

```text
Option+V
```

The installer downloads the latest `dedent-paste` executable from GitHub Releases, installs it to:

```text
~/.local/bin/dedent-paste
```

It also adds the `Option+V` rule to your active Karabiner-Elements profile and backs up your Karabiner configuration before changing it.

## Permissions

Karabiner-Elements may need macOS Accessibility permission to trigger paste through System Events.

Open:

```text
System Settings > Privacy & Security > Accessibility
```

Then make sure Karabiner-Elements is allowed.

## Related links

- [Karabiner-Elements](https://karabiner-elements.pqrs.org/)
- [Karabiner-Elements manual](https://karabiner-elements.pqrs.org/docs/)
- [Karabiner complex modifications](https://karabiner-elements.pqrs.org/docs/manual/configuration/configure-complex-modifications/)
- [GitHub Releases for dedent-paste](https://github.com/doggy8088/dedent-paste/releases)
- [macOS Accessibility permission guide](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)

## More information

- Website: [dedent-paste GitHub Pages](https://doggy8088.github.io/dedent-paste/)
- Development notes: [DEVELOPMENT.md](DEVELOPMENT.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)
- License: [MIT](LICENSE)
