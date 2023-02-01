# Hardware setup
A useful way to use midi-patch-changer is running on a Raspberry Pi with a small touch screen.
The following instructions describe how I set it up, which worked successfully for me but otherwise comes with no guarentees or support.

## Parts required
- [Raspberry Pi 3B+](https://core-electronics.com.au/raspberry-pi-3-model-b-plus.html) (other models may work but have not been tested, let me know if you try them and they work for you)
- [3.5" touchscreen hat](https://core-electronics.com.au/3-5inch-display-module-touch-lcd-with-stylus-for-raspberry-pi-3.html)
- [Case with screen cutout](https://www.ebay.com.au/itm/323709440552) or [3d print one](https://www.thingiverse.com/thing:4433642)
- 16GB+ microSD card
- PSU for Raspberry Pi (5V 2.5A)
- USB cables to connect to MIDI keyboards as required

## SD card setup
1. On another computer with a microSD card reader, install [Raspberry Pi Imager](https://www.raspberrypi.com/software/)
2. Insert microSD card into reader and run Raspberry Pi Imager
3. Follow instructions to flash microSD card with the following details:
- Raspberry Pi OS (64 bit) Full Desktop
- Enable SSH
- Set hostname, username and password
- Set WiFi & timezone details
- Add WiFi configuration of your home network (so you can configure it remotely)

## Assembly instructions
1. Put Raspberry Pi board into base of case
2. Connect touchscreen hat to GPIO pins on board (line up top corner pins with top corner of touchscreen so that the screen sits directly above the board)
3. Carefully put lid of base over screen and push until it clicks into place
4. Insert microSD card through slot in the bottom of the case
5. Plug in PSU and turn on

## Software configuration
1. Once the Raspberry Pi has turned on and booted up, it should connect to the WiFi network you configured above, confirm this using `ping HOSTNAME`
2. Use Putty or SSH in terminal to SSH into the raspberry pi using the username and password you configured above
3. Install LCD driver (https://github.com/goodtft/LCD-show)
- `cd ~`
- `git clone https://github.com/goodtft/LCD-show.git`
- `chmod -R 755 LCD-show`
- `cd LCD-show/`
- `sudo ./LCD35-show 180`
- Wait for Raspberry Pi to reboot, then reconnect SSH
4. Add on-screen keyboard (https://pimylifeup.com/raspberry-pi-on-screen-keyboard/, https://forums.raspberrypi.com/viewtopic.php?t=325263#p1959851)
- `sudo apt update`
- `sudo apt upgrade`
- `sudo apt install matchbox-keyboard`
- `sudo nano /usr/bin/toggle-keyboard.sh`
- Enter the following:
```
#!/bin/bash
PID="$(pidof matchbox-keyboard)"
if [  "$PID" != ""  ]; then
  kill $PID
else
 matchbox-keyboard &
fi
```
- Press Ctrl+X and Y to exit & save
- `sudo chmod +x /usr/bin/toggle-keyboard.sh`
- `sudo nano /usr/share/raspi-ui-overrides/applications/toggle-keyboard.desktop`
- Enter the following:
```
[Desktop Entry]
Name=Toggle Virtual Keyboard
Comment=Toggle Virtual Keyboard
Exec=/usr/bin/toggle-keyboard.sh
Type=Application
Icon=matchbox-keyboard.png
Categories=Panel;Utility;MB
X-MB-INPUT-MECHANISM=True
```
- Press Ctrl+X and Y to exit & save
- `cp /etc/xdg/lxpanel/LXDE-pi/panels/panel /home/pi/.config/lxpanel/LXDE-pi/panels/panel`
- `nano /home/pi/.config/lxpanel/LXDE-pi/panels/panel`
- Enter the following:
```
#APPEND
Plugin {
  type=launchbar
  Config {
    Button {
      id=toggle-keyboard.desktop
    }
  }
}
```
- Press Ctrl+X and Y to exit & save
- `mkdir ~/.matchbox`
- `nano ~/.matchbox/keyboard.xml`
- Enter the following:
```
<?xml version="1.0" encoding="UTF-8"?><keyboard><options></options><layout id="default keyboard"><row><key fill="true"><default display="Esc" action="escape" /></key><key><default display="`" /><shifted display="~" /></key><key><default display="1" /><shifted display="!" /></key><key><default display="2" /><shifted display='@' /><mod1    display="½" /></key><key><default display="3" /><shifted display="#" /><mod1    display="¾" /></key><key><default display="4" /><shifted display="$" /></key><key><default display="5" /><shifted display="%" /></key><key><default display="6" /><shifted display="^" /></key><key><default display="7" /><shifted display="&amp;" /></key><key><default display="8" /><shifted display="*" /></key><key><default display="9" /><shifted display="(" /></key><key><default display="0" /><shifted display=")" /></key><key><default display="-" /><shifted display="_" /></key><key><default display="=" /><shifted display="+" /></key><key fill="true"><default display="Bksp" action="backspace"/></key></row><row><key fill="true"><default display="Tab" action="tab"/></key><key obey-caps='true'><default display="q" /><shifted display="Q" /></key><key obey-caps='true'><default display="w" /><shifted display="W" /></key><key obey-caps='true'><default display="e" /><shifted display="E" /></key><key obey-caps='true'><default display="r" /><shifted display="R" /></key><key obey-caps='true'><default display="t" /><shifted display="T" /></key><key obey-caps='true'><default display="y" /><shifted display="Y" /></key><key obey-caps='true'><default display="u" /><shifted display="U" /></key><key obey-caps='true'><default display="i" /><shifted display="I" /></key><key obey-caps='true'><default display="o" /><shifted display="O" /></key><key obey-caps='true'><default display="p" /><shifted display="P" /></key><key><default display="{" /><shifted display="[" /></key><key><default display="}" /><shifted display="]" /></key><key fill="true"><default display="\" /><shifted display="|" /></key></row><row><key fill="true"><default display="Caps" action="modifier:caps"/></key><key obey-caps='true'><default display="a" /><shifted display="A" /></key><key obey-caps='true'><default display="s" /><shifted display="S" /></key><key obey-caps='true'><default display="d" /><shifted display="D" /></key><key obey-caps='true'><default display="f" /><shifted display="F" /></key><key obey-caps='true'><default display="g" /><shifted display="G" /></key><key obey-caps='true'><default display="h" /><shifted display="H" /></key><key obey-caps='true'><default display="j" /><shifted display="J" /></key><key obey-caps='true'><default display="k" /><shifted display="K" /></key><key obey-caps='true'><default display="l" /><shifted display="L" /></key><key><default display=";" /><shifted display=":" /></key><key><default display="'" /><shifted display="&#34;" /></key><key fill="true"><default display="Enter" action="return"/></key></row><row><key fill="true"><default display="Shift" action="modifier:shift"/></key><key obey-caps='true'><default display="z" /><shifted display="Z" /></key><key obey-caps='true'><default display="x" /><shifted display="X" /></key><key obey-caps='true'><default display="c" /><shifted display="C" /></key><key obey-caps='true'><default display="v" /><shifted display="V" /></key><key obey-caps='true'><default display="b" /><shifted display="B" /></key><key obey-caps='true'><default display="n" /><shifted display="N" /></key><key obey-caps='true'><default display="m" /><shifted display="M" /></key><key><default display="," /><shifted display="&lt;" /></key><key><default display="." /><shifted display="&gt;" /></key><key><default display="/" /><shifted display="?" /></key><key fill="true"><default display="Shift" action="modifier:shift"/></key></row><row><key fill="true"><default display="Ctrl" action="modifier:ctrl"/></key><key><default display="Alt" action="modifier:alt"/></key><key width="12000"><default display=" " action="space" /></key><key><default display="@" /><shifted display="'" /></key><key><default display="↑" action="up" /></key><key><default display="↓" action="down" /></key><key><default display="←" action="left" /></key><key><default display="→" action="right" /></key></row></layout></keyboard>
```
- Press Ctrl+X and Y to exit & save
5. Disable screen blanking (https://pimylifeup.com/raspberry-pi-disable-screen-blanking/)
- `sudo raspi-config`
- Select "Display Options" > "Screen Blanking" > "No" > "Ok" > "Yes"
- Exit raspi-config tool
6. Install midi-patch-changer
- `cd ~`
- `sudo apt-get install libfontconfig libfontconfig1-dev`
- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- `git clone https://github.com/davidlang42/midi-patch-changer.git`
- `cd midi-patch-changer/`
- `cargo build --release`
7. Add template patch files
- `mkdir ~/Patches`
- `cd ~/Patches`
- `ln -s /home/pi/midi-patch-changer/templates/roland_jp08 roland_jp08`
- `ln -s /home/pi/midi-patch-changer/templates/roland_gokeys roland_gokeys`
8. Add shortcut to taskbar
- `nano ~/.local/share/applications/midi-patch-changer.desktop`
- Enter the following:
```
[Desktop Entry]
Name=MIDI Patch Changer
Path=/home/pi/
Exec=./midi-patch-changer/target/release/midi-patch-changer Patches
Comment=
Terminal=false
Icon=gnome-panel-launcher
Type=Application
```
- Press Ctrl+X and Y to exit & save
- `nano /home/pi/.config/lxpanel/LXDE-pi/panels/panel`
- Enter the following:
```
#INSERT INTO Plugin{type=launchbar;Config{...}}
    Button {
      id=midi-patch-changer.desktop
    }
```
- Press Ctrl+X and Y to exit & save
9. Configure appearance settings
- Plug a USB keyboard into the raspberry pi (temporarily)
- Using the stylus (or finger), open main menu in the top left corner
- Select menu Preferences > Appearance Settings
- Select tab Taskbar > Size
- Change to Small or Medium by holding mouse down with stylus and using arrow keys on the keyboard & enter to change
- Click OK (or press tab until focussed and press enter)
10. Make midi-patch-changer start on boot
- `mkdir ~/.config/autostart`
- `cd ~/.config/autostart`
- `ln -S midi-patch-changer.desktop /home/pi/.local/share/applications/midi-patch-changer.desktop`
- `sudo reboot`
11. Enjoy your Raspberry Pi MIDI patch changer!
- If you have any suggested improvements, raise an [issue](https://github.com/davidlang42/midi-patch-changer/issues) or submit a [pull request](https://github.com/davidlang42/midi-patch-changer/pulls)
- If you find this useful, consider [buying me a coffee](https://ko-fi.com/davidlang42)