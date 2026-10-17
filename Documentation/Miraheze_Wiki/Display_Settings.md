# Display Settings

Source: https://toseven.miraheze.org/wiki/Display_Settings

One unique feature with ToSeven motors is that most of the motor parameters can be altered from the display. Other motors do not have this option, or they require connecting to a laptop and using a special software tool. However, there are a lot of parameters, and ToSeven has not yet been forthcoming with an easy-to-understand guide. The following is a work in progress but will hopefully help new owners with a starting point at least.

The T24 Display will be used. Other Displays should be similar with minor variations.

== How to use the T24 Display. ==
The T24 manual can be downloaded [https://to7motor.com/wp-content/uploads/2024/05/T24-Display-Maunual-240325-.pdf here]. There is a lot of information shown on the display as summarised here:
[[File:Picture173.png|frame|none]]

When you turn the display on for the first time by long pressing the power button (23), you should see “Calibrating, don’t tuch pedal”, followed by “Success”. This will not happen every time thereafter. It may only need to recalibrate after weather changes, settings changes, firmware updates and so on.

Long pressing the up button (20) will turn on lights if you have them connected. 

Hold the down button (22) for 2 seconds to activate “walk mode”. Let go of the button to deactivate. This can come in handy when you need to walk the bike for whatever reason. Note that it will take off immediately so be ready for it. Also, the speed it goes is relevant to the gear it’s in.

Long pressing the down button (22) will also turn on the Cruise Control if you’re moving. Hitting the brakes, pedalling, or pressing any button should deactivate it again. (Note, I couldn’t make this work so it may have been disabled in later firmware versions or there is some trick to it that I couldn’t figure out).

Pressing the “I” button (21) will change between Eco, City and Sport modes. See “[[#Controller|Advanced Settings]]” for more detail on what these do and how to set them up.

The up and down buttons will change the power level. These are also customisable. See “[[#Speed&Power|Advanced Settings]]”. Be aware that level 0 provides no power at all. Check this if nothing is happening when you start riding.

Short pressing the power button will change the info displayed on the bottom of the screen. A useful one here is “Test” which will display details of various parameters as you ride, which can help with fault diagnosis. Short press the power button to scroll through them and then hit the up button to exit. Another useful one is “Info”. Hitting the down button for this option will display the firmware versions of the Display and Controller.

To reset the timer, hold the power and down button together. Watch the screen with this one and let go of the buttons as soon as a reaction is seen. Holding them down for too long may reset everything. It may also be a bug and [[Troubleshooting#Display randomly resets|could randomly reset everything]].

== Main Settings == 
To get into the Main Settings menu, long press the Power button to start the system. As it begins to start, immediately long-press the arrow up button and the power button together until a prompt to enter a PIN appears. Then enter 1234 using the up and down buttons to change the number, and short press the power button to move to the next number. Here you will find these options ('''some options may or may not appear depending on the firmware version'''):

=== Unit ===
* Use the up and down buttons to change between Metric (kilometres) and “British” (miles). (I don’t know why they didn’t call this “Imperial” or “USA”).

=== Rim Size ===
* Use the up and down buttons to select your wheel size. The default is inches (24”, 26” etc). 
For fat tyre bikes, the following guidelines can be used:
:* 20 x 4.0 1605 mm ~23 inch
:* 24 x 4.0 2000 mm ~26 inch
:* 26 x 4.0 2285 mm ~29 inch / 700C
However, in later firmware versions (since October 2024) there is now a “RimSizeUnit” under “Device” which allows changing to mm for a more accurate speed reading. An easy way to determine a tyre’s diameter is to use the ISO/ETRTO number which is usually printed on the side.
[[File:Picture174.png|frame|none]]
The formula is 2 x tyre width + bead seat diameter x pi (3.142). In this example, 40 is the tyre width and 622 is the bead seat diameter, so: ((40 x 2) + 622) x 3.142 = 2206mm, (the number to enter as the Rim Size).

Note that the incremental units (i.e. go up or down in units of 1, 5, 10 etc) can be changed by holding the up or down button for a few moments.

=== Light ===
* Switch between Auto, Manual or On if you have lights connected.

=== AutoPowerOff ===
* Changes the amount of time the system will wait when the bike is doing nothing before it switches itself off.

=== Device === 
Selecting this one will give you the following options:

* '''Throttle''' - Enable or Disable (If disabled, the throttle works as walk assist, cutting out at 6km/h)
* '''PedalSensor''' - Enable or Disable (Later firmware. Should be enabled or the motor won’t work)
* '''PASStartCount''' - Enter the number of starter magnets up to 6. (Determined by manufacturer. Leave at the default of 1).
* '''TorqueSensor''' - Enable or Disable. When disabled, the motor uses cadence sensing (PAS) only.
* '''RimSpeedSensor''' - Enter the number of magnets attached to the wheel, which would normally be 1.
* '''ShiftSensor''':
:- Activate (enables or disables the shift sensor)
:- Delay (allows altering the amount of time the shift sensor cuts power)
:- Mode (how much the power is reduced to when the shift sensor is engaged) 
:''Note that the DM02 doesn’t come with an option to connect a shift sensor so this can be ignored if you have a DM02. Simply decrease pedal effort and it will shift easily and safely, even on internally geared hubs. Alternatively, lightly squeeze one of the brake levers just enough to activate the brake cutoff, then release it again after the gear change. Back pedalling will also instantly turn the motor off, which is great for internally geared hubs. It also works for derailleurs, but not if the chain stops completely since the chain needs to be moving to change gears. There is a delay, though, where the chain will continue moving but without any real power, allowing for a safe change.''
* '''CranksetTeeth''' - Enter the number of teeth on your chainring.
* '''CassetMaxteeth''' - Enter the number of teeth on your largest cassette cog.
* '''CassetMinTeeth''' - Enter the number of teeth on your smallest cassette cog.
:(To calculate these for internally geared hubs, see “[[Chains,_chainrings_and_chain_lines#Do_mid-drives_work_with_Rohloffs_and_other_geared_hubs|Do mid-drives work with Rohloffs and other geared hubs]]”).
* '''BrakeLight''' - if you have a brake light connected, options are: 
::- Disable (Brakes flashing)
::- Enable (Half on when not braking, full when braking) 
::- PWM - (All brakes are on)
* '''SpeedSensorFail''' - Choose from Speed Unlimit or Speed Limit. If set to Speed Limit, the motor will stop when no signal is received from the speed sensor. Choosing Speed Unlimit [[Troubleshooting#Dragging sensation when speed limit is exceeded|can also smooth out the dragging sensation]] some people experience when the speed limit in a given level is exceeded. 
* '''Battery''' (located here in later firmware versions from October 2024. Settings are the same as in older firmware versions in the [[#Battery|Advanced Settings]]).
* '''RimSizeUnit''' - (Later firmware versions). Set the wheel diameter in millimetres for a more accurate speed reading. See “[[#Rim Size|Rim Size]]” above for more info.

=== Password ===
Selecting this one will give you the following options:
* ModifyPassword - Change the PIN
* PasswordMode - Change the Password mode to one of these options: 
::- Menu (required to access the main menu)
::- None (No password required to access the Main menu. Note this does not disable it to access the Advanced menu)
::- PowerOn (requires a PIN when starting up, perhaps as an anti-theft measure)

=== Save ===
Saves any settings just altered, then shuts down. ('''Don’t forget to do this after making changes!''')

=== WithoutSave ===
* Discards any settings just altered, then shuts down.

=== FactoryDefault ===
* Restores everything to factory defaults. 
:If this doesn’t work, a workaround may be [https://discord.com/channels/713249006129512448/1093198123821694976/1385086778679230546 the following procedure] - click the power button, let go immediately, click again and hold for 1s, let go and then click again and hold until it turns on and says receiving.

== Advanced Settings ==
'''The information below is meant as a guide only, based on information gathered from the High Voltage Light Electric Vehicles Discord channel which you can join [https://highvoltagekits.com/discord here]. Some settings are “best guess” or conjecture. Be aware that while the settings here are assumed to be fine, it is possible that some might exceed the manufacturer’s recommendations and void warranty. Running at higher speeds and power levels may also increase heat and wear & tear. Use at your own risk.'''

=== How to Access the Advanced Settings menu ===
While in the Main Settings menu, scroll to “Unit” then long press the up and down buttons together. Then enter the password 1234 using the up and down buttons to change the number, and short press the power button to move to the next number.

'''Note: if this does not work, it has most likely been disabled by the manufacturer.''' All motors purchased after October 2024 have these settings disabled. Updating to some controller firmware versions and/or [[Firmware#Locked Displays|Display versions]] may also remove access. This is to cut down on warranty claims caused by people entering bad settings and damaging the controller and/or complaining about poor performance, and “to prioritize the motor's service life and user safety”. There is still a way to do it but this will void warranty if done without authorisation. Contact them for more information if you need to change something. 

Here you will find these options:

=== Application ===
Selecting this one will give you the following options:
* '''LevelCount''' - This allows you to have up to 8 levels of assistance. The default is 5.
* '''Cruise''' - Enable or Disable Cruise Control.

=== Speed&Power ===
This is the amount of Speed and Power for each level of assistance. What to set these on is individual preference and may require some experimentation to find what works for you. See [[Example Setups]] for reasonable starting points if you’re not happy with the factory defaults. Most people would want incremental settings here, beginning with low power and speed in the lower levels, up to 100% and no real speed limit at the highest setting. The cadence/PAS sensor mostly* uses the Spd** and Pwr values, and the EAssist values to a lesser (to no) extent. The torque sensor mostly* uses the EAssist and Spd** values, and the Pwr values to a lesser (or no) extent. 

<nowiki>*</nowiki>The amount may vary between firmware versions. In older firmware, Eassist settings had no effect on PAS mode, while power settings had no effect on torque sensor mode. In more recent firmware versions they seem to cross over a lot more.

<nowiki>**</nowiki>See [[#PAS|PAS > SpeedLevel]], and [[#TorqueSensor|TorqueSensor > SpeedLevel]] below for how Spd behaves. The values set in the Speed&Power levels tend to be vague and imprecise on their own.
 
Note also that there are also 3 modes, Eco, City and Sport, that govern the amount of assistance available in each of these levels as a group. See “[[#Controller|Controller]]” below.

EAssist levels work a little differently. Unlike the Pwr levels, these aren’t a power cutoff value as such. With the torque sensor enabled, they function more like the amount of effort you have to put in to reach the maximum that the motor can produce in the mode you are in (Eco, City or Sport). For example, if you have Level 1 EAssist set at 20%, it will provide 20% of its power when it senses pressure on the pedals, then ramp up all the way to the maximum as you put more pressure down. Likewise, if you have a level set at 80%, you get 80% of its power initially, ramping up to the maximum with very little extra effort. Lower values mean you have to work harder for the reward, and vice versa. Note that it could be argued that there is little point setting an Eassist level to 100% since it then behaves like PAS, providing full power as the pedals start turning with nothing extra when more pressure is applied. However, that’s not necessarily a bad thing for many people who would only use this setting when they want maximum power with minimum effort.*

<nowiki>*</nowiki> Doug disagrees with the above paragraph somewhat, and provided the diagram below based on his assessment.
[[File:Eassist1.jpg|frame|none]]

=== Battery ===
''Note: In more recent firmware versions, the battery settings are found in the Main Settings under “[[#Device|Device]]”.''

'''<span style="color:red">Configuring the battery is essential to prevent damage to the motor and also to gain the best performance'''.</span> See "[[General FAQs#Should you configure the motor's voltage setting|Should you configure the motor's voltage setting]]".

Selecting this one will give you the following options:

* '''Type''' - choose your battery voltage - 36 volts or 48 volts or “customize” if using something other than 36 or 48 volts. ('''Be aware''' that setting values that are too different to the “Type” value (e.g. selecting 36V and then entering the 48V values in the advanced battery settings) may stop the motor from working until the settings are changed to more closely match, and it may even [[General FAQs#Should you configure the motor's voltage setting|irreversibly damage the motor]]).
* '''Standard Voltage''' - adjust the voltage above in increments of 1.
* '''LowCutVoltage''' - To protect the battery, this setting will shut everything down if the battery falls to this level. The battery’s BMS (Battery Management System) will also cut out if it falls below a certain voltage*, so this is just an extra precaution.
 
<nowiki>*</nowiki> Battery BMS is normally set at 2.8V/cell which works out to the following values:
* 28V for 36V battery 
* 36.4V for 48V battery 
* 39.2V for 52V battery 
* 44.8V for 60V battery 
* 56V for 72V battery

If you don’t want the motor cutting out when there is still some juice in the tank, then use these values for LowCutVoltage, but if you want to be more conservative, set at ~2-4V above these values. If you’re running a battery without a BMS (which is ill-advised), definitely go ~2-4V above.

There are also some more (slightly hidden) settings here if you find the display is not showing the correct voltage. For example, showing less than 100% when it is fully charged. While in the Battery menu, long press the up and down buttons together. (see also [https://youtu.be/ng_2_noAkY0 this video]). This will bring up a new screen where the battery calibration can be fine-tuned. Select each % to adjust as necessary. A properly functioning battery and charger should result in the voltages in the chart below.
 
Note that the incremental units (i.e. go up or down in units of .1, .5, 1, 10 etc) can be changed by holding the up or down button for a few seconds.
[[File:Picture175.png|frame|none]]
Thanks to Atak_Snajpera for putting this chart together. For the ODS file of the above chart, [https://www.mediafire.com/file/6bqb9o0j8c9ikvg/VoltageScalling.ods/file click here].

Note that these are '''resting''' voltages and it will sag below these numbers under load. Also, if you compare the 0% values to the BMS cutoff values above, they are several volts higher. The reason for this is that after these 0% values are reached, there is a sudden nonlinear (waterfall) drop in voltage. (See chart below). You are not getting many Wh after that point. Maybe 3-4% max, so it is good to have that few percent hidden below 0% on display. 
[[File:Picture176.png|frame|none]]

However, if you have a large battery, then you may find you still have quite a few miles left in the tank while the display is showing 0%. If that is unsuitable, you can try dropping 1-2 volts from the 80-0% values, or use the numbers in the table below. These are plotted to be linear and with a lower 0% value. 
{| class="wikitable" style="width:600px; text-align:center;"
|-
! style="width:16%;" | Battery
! style="width:16%;" | 36V
! style="width:16%;" | 48V
! style="width:16%;" | 52V
! style="width:16%;" | 60V
! style="width:16%;" | 72V
|-
! 0%
| 30.0
| 39.0
| 42.0
| 48.0
| 62.0
|-
! 20%
| 32.2
| 41.9
| 45.1
| 51.5
| 66.0
|-
! 40%
| 34.4
| 44.7
| 48.2
| 55.0
| 70.0
|-
! 60%
| 36.6
| 47.6
| 51.2
| 58.6
| 74.0
|-
! 80%
| 38.8
| 50.4
| 54.3
| 62.1
| 78.0
|-
! 100%
| 41.0
| 53.3
| 57.4
| 65.6
| 82.0
|}

However, if the battery tends to cut out too early using these figures, download [https://docs.google.com/spreadsheets/d/1n_VjkqcA8sP-HxkLe_euqDq02F93718ESLPVsEYlmHo/edit?usp=drive_link this spreadsheet] and raise the 0% values by 1 or 2V (the 20-80% values will automatically update to match) until you find a happy medium.

'''Note''': the “Test” menu from the display will show battery voltage. After connecting a freshly charged battery, go to this menu (on the T24, press the power button repeatedly until “test” is displayed at the bottom, then hit the up button to enter), then use this value for 100% battery in the chart above.

=== Controller ===
Selecting this one will give you the following options:
* '''MaxCurrentECO'''
* '''MaxCurrentCITY'''
* '''MaxCurrenSPORT'''
These three control the maximum amps available in Eco, City, and Sport modes. These modes can be switched between on the display by pressing the “I” button, the idea being to have modes with better battery range and lower power when needed, and a Sport mode when you want to cut loose. On the 500-watt or lower DM02, having three modes could be considered unnecessary, and it would be better to just have one mode that does it all, controlling the amount of power in each level instead. This was Ian’s plan mentioned in the “[[Example Setups#One Mode to Rule Them All|the one mode to rule them all]]” setup example. However, it doesn’t hurt to have them there as well. The main thing is not to exceed the maximum amps that the controller can support since it may cause it to overheat and/or possibly damage it. For the 500-watt version with a 48V battery this is 18 amps.* (15 amps for the 250- & 350-watt versions). With a 36V battery it can go as high as 20 amps. From there you want it to be progressive, so for Eco mode for the 500-watt motor with a 48V battery you might start with 8 amps, 13 for City, and 18 for Sport. Or if you want to be more conservative, 8, 12 & 16 amps, or for a wider range of difference, 6, 12 & 18 amps. (Note that the display will not allow going below 5 amps).

<nowiki>*</nowiki> The manufacturer has since revised this recommendation to 14.2 amps with a 48V battery. See “[[General FAQs#What are the maximum safe amps for the 500W DM02|what are the maximum safe amps for the DM02]]”. 

Note that whether using older firmware where 18 amps is possible, or with newer firmware set at 14.2 amps, sustained effort at maximum amps (i.e., slogging away at low RPM up a massive hill), or in hot conditions, can cause the controller to reach its maximum temperature. If that happens, its temperature control measures will step in and reduce power or shut it down until it cools sufficiently. To keep the motor at its happiest, remember to change down gears and spin when you feel it bogging down.

=== Motor ===
Selecting this will give you the following options. <span style="color:red">'''These should be left alone unless you know what you’re doing'''</span>:
* '''PolePairs''' - Magnet pairs in the motor, should not be changed.
* '''Reduction Ratio''' - Reduction ratio of gearing, should not be changed
* '''AngleOffset''' - Likely to do with the FOC controller angle, should not be changed
* '''MotorConstant''' - not sure what this does. Leave alone.
* '''StartMinVoltage''' - not sure what this does. Leave alone.
* '''StartSpeed (rpm)''' - not sure what this does. Leave alone.
* '''StartMaxCount''' - not sure what this does. Leave alone.
* '''StartValue''' - not sure what this does. Leave alone.
* '''Test motor''' - Warning, make sure the rear wheel is off the ground with this one as the motor will start immediately. This is to test the above parameters. 

=== PAS ===
Selecting this will give you the following options:

* '''Type''' - (in some older firmware versions). Options are Auto, High, Low and Disable. Disabling it causes [https://discord.com/channels/713249006129512448/1093198162417688656/1495983456100945970 bizarre power on/off behaviour] so leave at auto. 
* '''MagnicCount''' - (in some older firmware versions) now moved to "RimSpeedSensor" under Device in the main settings
* '''StartupCount''' - the number of PAS starter magnets (determined by manufacturer)
* '''KeepTime(ms)''' - 200ms up to 1000ms. This is the length of time the motor continues after you stop pedalling. The default is 600ms (a touch over half a second) which works quite well. Lowering it to 200ms reduces run-on even more but can also reduce responsiveness and cut out completely at low speeds or if cadence drops below 60rpm. I also experienced these issues with it set to 400ms.
* '''Strength (rpm)''' - best guess is that it is supposed to regulate the power curve according to cadence. Default is 10. Lower values tend to make the power ramp up jerky and unpredictable.
* '''SpeedMode''' - Choose from Speed_Level, Speed_MaxLevel, and NoSpeedLimit. 
:- If Speed_Level is selected, it will cut out at the maximum speed set for each level. i.e. if you’re in level 2 and have it set to 25kph, it will stop assisting when it reaches 25kph.
:- If Speed_MaxLevel is selected, the motor will keep accelerating up to the maximum Speed level set in the Speed & Power settings.
:-  NoSpeedLimit behaves as expected.
:(''My preference is Speed_Level since it can be difficult to maintain a set speed in PAS mode (torque sensor disabled) when needed. Note that when you exceed the speed limit in a given level and the motor cuts out, it can take a few pedal strokes for the motor to engage again once you drop below the limit again'').
* '''MaxSpeed (rpm)''' - set the maximum cadence (rpm) you want the motor to keep applying assistance up to. After that, it will cut out.
* '''CurrentOffset (a)''' - This is the amount of power applied when the PAS sensor is triggered after you start pedalling. The default is 5 but if you find it too aggressive, dropping as low as 0 amps can help.

See also “[[Example Setups#The Hauler|The Hauler]]” setup example for suggested settings to get the best out of PAS mode.

=== Throttle ===
Selecting Throttle in the Advanced menu will give you the following options:
* '''Activate''' - Enable or Disable the throttle. (If disabled, the throttle works as walk assist, cutting out at 6km/h)
* '''SpeedMode''' - Choose from 0-12, with 12 “Scooter mode” being the highest. 
The manual has the following table for SpeedMode, explaining that the throttle behaves differently depending on whether or not PAS is in operation. i.e. you’re pedalling or not when you engage it.
[[File:Picture177.png|frame|none]]
* The "PAS not working" column is how it behaves while not pedalling. “Null not working” means the throttle doesn’t work at all if not pedalling as well.
* The "After PAS work" column is how it behaves while pedalling as well.
* “Level (Gear speed)” means the throttle speed follows your current assist level. For example: if assist level 1 is set at 10 km/h → throttle also maxes at 10 km/h.
* “Full (maximum gear speed)" means the throttle goes straight to the top speed of the highest assist level, no matter what level you're actually in. (This is still tied to the mode you are in - Eco, City or Sport).
* “Full” vs “Full2”, at a guess I think Full means it provides the same power as the level you're currently in, while Full2 should give your top level power. (This need confirmation by someone who has tested it)

'''Note:''' Some of these may have changed slightly in more recent firmware versions. For example, in the latest firmware versions, the default is Mode 7 which is 40 km/h pedalling (i.e. Level 5) or 32 km/h for throttle only. Be aware of that when you install a new motor, update firmware or perform a factory reset. For strict EPAC countries, this needs to be changed to Level 3, or remove the throttle completely depending on the local law. Remember that walk mode (holding the down button) will also give you 6kmh without pedalling, so you don't really need the throttle connected to get a boost off the line. Level 5 or 9 may possibly be legal as well in some countries, but be careful! Technically, these fulfill the requirement of "must cut out at 6kmh if not pedalling" but some cops and/or insurance investigators in the event of a crash may not see it that way.

=== TorqueSensor ===
Selecting this will give the following options. Most of these should be left alone unless you know what you’re doing:
* '''Activate''' - Enable or Disable (also available in the main menu under “Device”)
* '''Mode''' - Choose from Safe or Sensitive. Safe is the default and uses a combination of pressure on the pedals and pedal rotations. Sensitive focuses more on pressure on the pedals which will reduce lag after coasting but it can also feel too aggressive for some people. Reducing StrengthValue can help reduce this.
* '''ZeroVol(mv)''' - Torque sensor voltage when idle. Default is 700mv (0.7V). This is the minimal force required to activate the motor. Higher value = more force on pedal has to be applied. Increase the value if you want less assist from the very start.
* '''StartupVol(mv)''' - Startup voltage. The necessary increase in voltage that must be applied when the pedals are pushed for the motor to provide power. Default is 200mv.
:- [https://discord.com/channels/713249006129512448/1093198162417688656/1300889834851926108 One owner] has been experimenting with the above two settings and found that setting ZeroVol to 1100mv and StartupVol to 100mv gave much smoother acceleration, reduced the ghost pedalling effect significantly, and the motor reduced power much earlier which made gear changes safer.
[[File:Picture178.png|frame|none]]
* '''CurrentOffset (a)''' - This is the amount of power (amps) applied when the torque sensor is triggered when pressure is applied to the pedals. If you find it too aggressive, lowering it can help.
* '''Filter''' - The “ramp time” or the time it takes to accelerate to the desired power. Lower value = faster acceleration, higher = slower, smoother acceleration.  Increase for a more natural bicycle feel.
* '''StrengthValue''' - How much power the motor provides in response to how much pressure is applied to the pedals. Works in conjunction with, and affects, EAssist level. Default is 10. Reducing this number to 3 or 4 is reported to reduce surging and create a smoother startup but it also requires more muscle to get the same response from the motor which can make it feel less responsive and lower powered. Basically, it depends how easy you want it to be. Lower values mean more muscle is required.
* '''SpeedMode''' - Choose from Speed_Level, Speed_MaxLevel, and NoSpeedLimit. 
:- If Speed level is selected, it will cut out at the maximum speed set for each level. i.e. if you’re in level 2 and have it set to 25kph, it will stop assisting when it reaches 25kph. 
:- If Speed_MaxLevel is selected, the motor will keep accelerating up to the maximum Speed level (default Level 5) set in the Speed & Power settings. 
:- NoSpeedLimit will ignore the speed values set in the Speed & Power settings and keep accelerating as fast as the motor's power will allow in each level.
* '''Keeptime''' - unlike this option in the PAS settings, lowering it below the default 2000ms (20x100ms) can make it feel much more lively and responsive, while raising it can reduce the rubber banding effect while pedalling. The highest value allowed is 5000 (50x100ms). Lower value = faster deceleration of the motor when you reduce force on pedals.

=== TempSensor ===
Selecting this will give you the following options:
* '''MinTemp''' - starting point for overheating. (Default is 70°C)
* '''MaxTemp''' - maximum overheating point. (Default is 100°C)
When the motor reaches MaxTemp, the motor will drop 70% of its power until it cools sufficiently. This behaviour is different to older firmware versions as seen in [https://youtu.be/_EbKgmae1zI?si=jlmTt_YkKmHrEGuI&t=185 this video from 3:05]. Here he experienced a 70% loss of power when it hit the MinTemp level, which in his case was 60°C. Bear in mind this older version of the firmware also had added parameters to enable and disable the Controller TempSensor and also set the amount of throttling that would occur. These options have since been removed and the MinTemp now defaults to 70°C. This value possibly has a bearing on cold weather conditions as it does not seem to be a factor in when the motor reduces power now. For example, [https://youtu.be/u_uWuyOM7ok?si=_B5J5zjinArqkoIi Ian mentions in this video] that he hits 83.9°C for several kilometres but makes no mention of the power stepping down.
[[File:Picture179.png|frame|none]]

The highlighted options are no longer available, but it is safe to assume they are still there but hard set by the factory.
 
[https://discord.com/channels/713249006129512448/1093198162417688656/1292851848612417558 Rarebird6969 from California Ebike reports]:

:''When the controller temp reaches the factory default temp of 100c, the motor will “roll back” decrease power to 30%…. you will feel it drop in power instantly.  This is the motor telling the rider “Hey man….I’m overheating here and I need a breather!”. As soon as the temp drops below 100c the motor will kick back up to 100%.''

:''It’s best practice to stop and let the motor cool off for a few minutes. Also evaluate the current riding conditions to assess why is the motor getting too hot…. are you in the proper gearing…..or is your gearing not low enough for the hill you may be climbing…. have you been using the throttle too much for a long period of time…. too much weight (cargo bikes) etc….. If you don’t stop to take a moment to let it cool, you will then get an intermittent power decrease that will give the rider the feeling the motor is broken.  It’s not broken, it’s just the controller doing what it was designed to do…. decrease power to help you save the motor before something breaks. This is BIG concept that a lot of riders ignore or just are not aware of how to monitor.''

:''If you are going to do some hard climbing or long throttle usage….. I would advise to use the display temp reading to help you see your current temp. On the T24:''

:''Click the power button 10times till you see “Test” and the ^ icon on the bottom of display. Then click the ^ button to access the display diagnostic readings….click the power button until you see the live “controller temp” reading. (Ignore the Ambient temp).''

:''If 100c is too high for you, the temp setting can be lowered to whatever you want in the “Advanced Menu” parameters under the last tab “Temp“.''

[https://discord.com/channels/713249006129512448/1093198162417688656/1293686985604464742 He has also tested this in the lab]:

:''I have pushed the DMO2 and the DMO1 Past 100 Celsius many times under testing and the throttle-back at 100 Celsius works and is a great indicator that I need to stop and let the motor cool off.''

:''I’ve pushed it far enough to blow multiple controllers and melt the nylon gear as well. The key to remember if you’re going to push the motor hard is to put the display on the screen that shows temperature so that you can monitor your temp as you ride.''

The manual also states that there should be a warning on the display if the controller or motor goes over temperature.
[[File:Picture12.png|frame|none]]

Olek from ToSeven had this to say [https://discord.com/channels/713249006129512448/1093198162417688656/1286271052883693645 in the Discord channel] which is also worth noting:

:''Heat is relative, the controller will throttle the motor. There are too many variables, you could overheat the motor at 15a if you put it on high gear. Large front ring and pedal at a low cadence. If you are running in the right gear at a very comfortable cadence, say 70 rpm upwards, then the motor overheating will be down to environmental temperatures. This cadence rule is a general rule of thumb and will apply to all mid drive motors. If you are struggling so is the motor, drop a gear.''

:''We have all of this lab tested at ambient temperatures of up to 45°C with and without replication of airflow cooling. The motor is reliable at 18a. If it gets too hot it will self-throttle.'' 

:''We have to allow for individual circumstances and use cases, please understand this. We cannot account for every possible scenario. That’s why there are user adjustments and automatic throttling. If you are in a very hot hilly environment you might want to adjust your max amps or change out your rear cassette.''
