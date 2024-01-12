# UI Requirements
*(WIP)*

## UI Elements

### Record Button
* Begin recording data from sensors
* Connects to handler on SBC
* Large and clearly visable for user

### Stop Button
* Stops recording data from sensors
* Large and clearly visable for user

### Data Plot
* Visual plot element for data
* Plot data in a clear and readable way
* Plots for each sensor

### Toolbar
* Switch between view data, download data, configure sensors

## Settings and Configuration
* Configure sensor sensitivity
* Turn sensors off and on
* Connect new sensors?
* Configure IP

## Size and Memory Usage
* Must be able to function on a Raspberry Pi 4
* Should be within 4 GB

## Security
* Sanitize inputs
* Rocket TLS configuration
    * Use self signed certs for development

## Frameworks
* EGUI
* Rust Rocket
