# RPI Motorsports SIDS

This repository contains the source code for the RPI Motorsports' Student ID Scanner (SIDS).

### Hardware Info

At RPI, the campus uses the following scanners
- RFIDeas RDR-6081AKU
- HID Global Proximity MAXIPROX
Given that the RPI cards operate on an HID protocol, a scanner compatible with scanning HID cards is required. For this
application, we use an Omnikey 5025CL, and all code is based around allowing specifically it to work.

### Cloning the Code

To clone the current master (stable) code, use the following:

`git clone https://github.com/PSMusicalRoc/Motorsports-SIDS.git`

As for the code in any given development branch, that would be the following (substitute `<branch_name>` with the name of the branch you'd like to clone!):

`git clone -b <branch_name> https://github.com/PSMusicalRoc/Motorsports-SIDS.git`


### Running the code

In general, the code can be run by typing the following into a command prompt:

`cargo run` or `cargo run --release` for the production-ready version.

When you're finished with the webserver, exit the program with `CTRL-C`.


## Authors

- Tim Bishop \<bishot3@rpi.edu> - Main Contributor, Repo Owner
