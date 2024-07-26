# Traktor Usage with Ableton Push 2

This repository provides programs, helper scripts, and resources for integrating [Traktor Pro 3](https://www.native-instruments.com/en/products/traktor/dj-software/traktor-pro-3/) with the [Ableton Push 2](https://www.ableton.com/en/push/).

## Features

- **Traktor Mapping**: Customized mappings for Ableton Push 2 to control Traktor Pro 3.
- **Display Driver**: Utilizes the Push 2 display for showing current effects and other functions with real-time feedback.
- **Resources**: A number of resources I found while researching which might be helpful


## Usage

Insall the api by replacing the traktor files in `C:\Program Files\Native Instruments\Traktor Pro 3\Resources64\qml\CSI\D2` with the files from `.\traktor_api\D2\`.

If you don't own a Traktor Kontrol D2:
    - Go to Preferences > Controller Manager
    - Below the Device dropdown, click Add… > Traktor > Kontrol D2

Load the traktor mappings from `.\traktor_mappings\complete` and start the `push2display` app with `cargo run`.