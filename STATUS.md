
| Functionality           | None  | Basic | Most  | Complete  |
| -------------           | :--:  | :---: | :--:  | :------:  |
| Callback framework      |       |       |       |     •     |
| Reference counting      |       |       |       |     •     |
| **Dialogs**             |
| Dialog                  |       |   •   |       |           |
| FileDlg                 |       |       |   •   |           |
| ColorDlg                |   •   |       |       |           |
| FontDlg                 |   •   |       |       |           |
| ProgressDlg             |   •   |       |       |           |
| Alarm                   |       |   •   |       |           |
| GetParam                |   •   |       |       |           |
| Message                 |   •   |       |       |           |
| LayoutDialog            |   •   |       |       |           |
| ElementPropertiesDialog |   •   |       |       |           |
| **Containers**          |
| Expander                |   •   |       |       |           |
| Fill                    |       |       |   •   |           |
| Frame                   |       |   •   |       |           |
| GridBox                 |       |   •   |       |           |
| HBox                    |       |   •   |       |           |
| Normalizer              |   •   |       |       |           |
| Radio                   |       |   •   |       |           |
| ScrollBox               |   •   |       |       |           |
| Split                   |   •   |       |       |           |
| Tabs                    |       |   •   |       |           |
| VBox                    |       |   •   |       |           |
| ZBox                    |   •   |       |       |           |
| **Standard**            |
| Button                  |       |   •   |       |           |
| Calendar                |   •   |       |       |           |
| Canvas                  |       |   •   |       |           |
| DatePick                |   •   |       |       |           |
| Label                   |       |   •   |       |           |
| Link                    |   •   |       |       |           |
| List                    |       |   •   |       |           |
| ProgressBar             |   •   |       |       |           |
| Text                    |       |   •   |       |           |
| Toggle                  |       |   •   |       |           |
| Tree                    |   •   |       |       |           |
| Val                     |   •   |       |       |           |
| **Menus**               |
| Item                    |       |   •   |       |           |
| Menu                    |       |   •   |       |           |
| Separator               |       |       |       |     •     |
| Submenu                 |       |   •   |       |           |
| **Other**               |
| Image                   |   •   |       |       |           |
| Clipboard               |   •   |       |       |           |
| Timer                   |   •   |       |       |           |
| Global attributes       |   •   |       |       |           |
| **Controls library**    |
| GLCanvas                |   •   |       |       |           |
| MatrixEx                |   •   |       |       |           |
| MatrixList              |   •   |       |       |           |

These I don't plan to implement, and I'm not sure I want in the library:

- Cbox (absolute positioning is bad, but if it was useful for a custom layout, I might implement it)
- BackgroundBox (backgrounds normally don't need changed)
- FlatButton (use a regular button that looks native)
- GetText (overlap with other dialogs, modal, and limited)
- ListDialog (modal and limited)
- MessageDlg (overlap with Alarm and non-customizable button text, which is usually worse UX)
- Sbox (I've never seen a program with one of these)
- Spin and SpinBox (text boxes support spin controls)

These I may implement, but at this time are lower priority:

- Plot
- Scintilla
- WebBrowser
