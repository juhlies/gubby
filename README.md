# gubby 🐰
### an le epic github "package manager"
<p>
  <sup><em><strong>ONLY</strong> supports windows-x86_64!!!!</em></sup>
</p>

- not intended for anyone other than myself and friends
- only really works on elite ebk CLIs coz that's what i made it for
- kinda bad it's my first attempt at anything in rust very youtube tutorial crutched
- `install` and `update` always prioritize stable releases if you want a prerelease just run `gubby install https://github.com/author/repo.git x.y.z-beta.w`

## installation
1. download latest release
2. extract to folder
3. open the folder
4. right click
5. choose "open in terminal"
6. run `gubby.exe self-install`
7. delete folder

## usage
**gubby.exe** [COMMAND]
| command | description | args |
| :---- | :---- | :---- |
| check | checks packages for updates | *optional*: `url`/`name` |
| config | opens gubby config file | `none` |
| install | installs a package | `url`, *optional*: `version` |
| list | lists installed packages | `none` |
| self-install | installs the gubby binary | `none` |
| self-upgrade | installs the latest version of gubby | `none` |
| uninstall | uninstalls a package | `url`, `version` |
| update | updates installed packages | *optional*: `url`/`name` |

## build from source
### requirements
- rust toolchain
- cargo
- windows-x86_64

1. `git clone https://github.com/juhlies/gubby.git`
2. `cd gubby`
3. `cargo build --release`
4. `target/release/gubby.exe self-install`
5. `cd .. && rmdir /s /q gubby`
6. `77777 nahh boi tuffff`