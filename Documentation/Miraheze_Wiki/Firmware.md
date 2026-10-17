# Firmware

Source: https://toseven.miraheze.org/wiki/Firmware

== Is a programming cable needed ==
While current motors should come with programming cables for both the controller and display (at least if purchased from ToSeven’s AliExpress store), older versions did not. The programming cables are needed to update firmware to take advantage of improvements and bug fixes. They are not needed to alter settings.

== How to check firmware version ==
After booting up, short press the power button to scroll through different settings at the bottom of the screen. When you get to “Info” short press the down button. However, there are some inconsistencies in the numbers displayed and this may not mean much. The only way to be 100% certain is to install a known firmware version.

== Is updating firmware necessary ==
Firmware updates often contain bug fixes and improvements, and is usually the first thing to try when you have a problem, but it can also sometimes introduce other problems as well. Unless you have experienced problems such as “[[Troubleshooting#What does the “Motor Blocked” error message mean |Motor Blocked]]” errors, [[Troubleshooting#Ghost pedalling or gradual loss of power with torque sensor enabled (DM02)|poor torque sensor behaviour]] or other issues that point to a software problem, it is advisable to leave well enough alone. Updating firmware on any electronic device also comes with a risk of [[Troubleshooting#What is a bricked controller |permanently bricking it]], and [https://discord.com/channels/713249006129512448/1093198123821694976/1348986809421664400 it does happen with ToSeven controllers as well], particularly with [https://discord.com/channels/713249006129512448/1093198123821694976/1382843864129208441 the DM01].

Firmware update decision flow chart.

[[File:Picture191.jpg]]

For Display firmware, there is usually no reason to update unless it has become unresponsive and possibly corrupted, or you need to rule it out as a possible issue. There has been no feedback that a particular Display firmware version does not work with a particular controller firmware version. Be aware also that [[#Locked Displays|more recent Display firmware versions may lock you out of the Advanced settings]].

== How to update the firmware ==
Firmware and instructions for the Controller and Displays are available [https://to7motor.com/progam-updating-instructions here]. However, there are a few things not mentioned to be aware of: 
* All files will need to be unzipped prior to installing. Do not just double click the zip file and drag & drop the contents. Some necessary files may remain hidden this way. Right click the zipped folder and choose “extract all”.  
* The December ‘24 Controller Operation Interface software requires [https://www.win-rar.com/ WinRAR] to unzip and the process is a little tricky. First double click the “.zip” file, then drag and drop the file ending in “.rar..x-rar-compressed” onto your desktop or wherever you want it. Then right-click that file and select “Open With” and choose WinRAR. Then drag and drop the file ending in “.exe” to your desktop or wherever you want it. Then double click that to install it on your PC. See also “[[#Locked Displays|Locked Displays]]” below.
* Some versions of the Controller Operation Interface software will not work to install some firmware versions. See [[#Controller firmware update tool versions|below]].
* Some firmware versions have incorrect file extensions. The controller firmware should end in .pack, while the display firmware should end in .hex. If they have ..bin on the end, this needs to be removed (renamed) prior to installing the firmware or the installation program will not work.
[[File:Picture168.png|frame|none]]
* When updating the controller, first turn the battery off when connecting the cable, then turn it on. If the PC cannot see it on the com port, try turning the battery off and back on again. (Note that the battery must be on for the process to work).
* Procedure wise, follow the instructions carefully. There are a couple of tricks to it. For example, step 6 when updating the Display firmware. The power button must be held down during the process. Once completed, the “success” message is in Chinese. When it appears you have 10 seconds to disconnect the cable or the process will start again. Disconnecting the cable during the process may corrupt and brick the Display, but it will usually be recoverable by repeating the process.
* After updating, most controller settings will go back to factory defaults and will need to be re-entered. e.g. DM01 battery will default to 48V, DM02 will default to 36V. It is a good idea to keep a written record of all settings prior to updating firmware.

== DM02 firmware versions ==
ToSeven does not make older versions readily available, but they are still hidden on their website at the following links ('''strikethrough means not recommended'''):

<s>'''Dec 14, 2023''' - [https://to7motor.com/wp-content/uploads/2023/12/TO7-DM02-K231214A-V6205C.zip K231214A-V6205C]</s>

<s>'''Mar 25, 2024''' - [https://to7motor.com/wp-content/uploads/2024/03/TO7-DM02-K240327A-V6205C.zip K240327A-V6205C]</s>

'''May 12, 2024''' - [https://to7motor.com/wp-content/uploads/2024/05/TO7-DM02-K240512A-V6206C.zip K240512A-V6206C] 

'''Jun 12, 2024''' - [https://to7motor.com/wp-content/uploads/2024/06/TO7-DM02-K240612B-V6306C.zip K240612B-V6306C] 

'''Jul 23, 2024''' - [https://to7motor.com/wp-content/uploads/2024/07/TO7L-DM02-K240723A-V6306C2.zip K240723A-V6306C2]

<s>'''Oct 16, 2024''' - [https://to7motor.com/wp-content/uploads/2024/10/TO7-DM02-K241016-V6212C.pack_.zip K241016-V6212C]</s> - To7 reports this one has a bug but did not say what it is.

'''Oct 31, 2024''' - [https://to7motor.com/wp-content/uploads/2024/10/TO7-DM02-K241031-V6303C.zip K241031-V6303C]
* Moves battery settings to the Main menu under “Device” and adds rim size in mm for more accurate speed and distance readings
.
<s>'''Nov 29, 2024''' - [https://to7motor.com/wp-content/uploads/2024/11/TO7-DM02-K241129A-V6400C.zip K241129A-V6400C]</s>: 
* Factory release notes [https://drive.google.com/file/d/13dCUvsOVVuoGTWAqf2_Qo9jHbEEMbZG8/view?usp=drive_link here].
* Fixes ‘motor blocked’ errors
* Improves torque sensor behaviour (see “[[Troubleshooting#Ghost pedalling or gradual loss of power with torque sensor enabled (DM02)|Ghost pedalling]]”)
* Amps are at different values but are hard set and cannot be changed (see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]). For more information about why this amp cap was necessary and what it does, see ToSeven's explanation [https://endless-sphere.com/sphere/threads/technical-update-dm02-500w-firmware-optimization.129704/ here].
* Speed is capped at 32kph throttle-only and 40kph while pedalling. These can still be changed if you have access to the advanced menu. (See see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* Does not remove access to the Advanced menu. If you had access before you will still access after. If you did not have access before (post Oct motors), you will still not have access
* Some reports of the software simply not working to install this version. In this case, the old updater may work whereas the new one won’t, or vice versa. See [[#Controller firmware update tool versions|Controller firmware update tool versions]]
* Some complaints of finding the torque sensor too aggressive in the lower levels and/or the speed limits in lower levels being too low or too high. (Everyone has their preference).
* It is not possible to roll back to an older version after installing this one
<s>'''May 21, 2025''' - [https://to7motor.com/wp-content/uploads/2025/05/TO7-DM02-K250402-V6400C.zip K250402-V6400C]</s>:
:'''''Manufacturer's release notes:'''''
* Fixes an issue where parameters were not being saved.
:'''''Other:'''''
* Reported to remove access to the Advanced settings with older T24 display firmware but possibly can be worked around with the latest T24 firmware.
* Amps are still hard set and cannot be changed (see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]). For more information about why this amp cap was necessary and what it does, see ToSeven's explanation [https://endless-sphere.com/sphere/threads/technical-update-dm02-500w-firmware-optimization.129704/ here].
* Speed is capped at 32kph throttle-only and 40kph while pedalling. These can still be changed if you have access to the advanced menu. (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* It is not possible to roll back to an older version after installing this one
'''Aug 1st, 2025''' - [https://to7motor.com/wp-content/uploads/2025/08/TO7-DM02-K250801-V6510C.zip K250801-V6510C]:
:'''''Manufacturer's release notes:'''''
* Resolved the controller malfunction that occurred after the system upgrade
* Enhanced the performance of the torque sensor for greater stability
* Further improved overall riding quality for a smoother and more comfortable experience.
:'''''Other:'''''
* [https://discord.com/channels/713249006129512448/1093198162417688656/1413428100045340672 Some reports] that this firmware version is causing more rapid heating of the controller which is causing even worse “[[Troubleshooting#Ghost pedalling or gradual loss of power with torque sensor enabled (DM02)|voltage drift]]” issues with the torque sensor. However, [https://discord.com/channels/713249006129512448/1093198162417688656/1416316682393092106 it worked very well for me] and pretty much cured these issues. With that being said, [https://discord.com/channels/713249006129512448/1093198162417688656/1416705195949297705 Ian has several old and new motors with old and new firmware that he is experimenting with] and has noted variations to how they behave with different firmware versions.
* [https://discord.com/channels/713249006129512448/1093198162417688656/1435914341420498975 One report] of not functioning below -8°C. 
* Controller amps are still hard set  (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]) For more information about why this amp cap was necessary and what it does, see ToSeven's explanation [https://endless-sphere.com/sphere/threads/technical-update-dm02-500w-firmware-optimization.129704/ here].
* Speed is capped at 32kph throttle-only and 40kph while pedalling. These can still be changed if you have access to the advanced menu. (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* It is not possible to roll back to an older version after installing this one

'''Feb 05, 2026''' - [https://to7motor.com/wp-content/uploads/2026/02/TO7-DM02-K260203-V6510C.zip K260203-V6510C]:
:'''''Manufacturer's release notes:'''''
* The latest DM02 update focuses on functional optimization, adding an option to manually disable the 6km/h Walk Assist. 
* Introduces a dynamic zero-point calibration algorithm. The controller no longer relies solely on the initial voltage at power-on, but instead tracks and corrects the sensor's "zero point" in real time during riding (when pedal pressure release is detected). This effectively eliminates signal offset caused by temperature rise at the logical level, thus preventing "ghost pedaling."
:'''''Other:'''''
* See also [https://endless-sphere.com/sphere/threads/technical-update-dm02-500w-firmware-optimization.129704/ Endless Sphere] where ToSeven discuss what they've been working on.

<s>'''May 14, 2026''' - [https://to7motor.com/wp-content/uploads/2026/05/TO7-DM02-K260511-V6510C.zip K260511-V6510C]:</s>
:'''''Manufacturer's release notes:'''''
* This version is mainly about hardware protection and stability. Our engineers reworked the current management layers to make the controllers much more resistant to voltage spikes and high-load stress, which significantly minimizes the chance of component failure.

'''May 26, 2026''' - [https://to7motor.com/wp-content/uploads/2026/05/TO7-DM02-K260526-V6510C.zip K260526-V6510C]:
:'''''Manufacturer's release notes:'''''
* As above. It is assumed that since this version came out so soon that the previous version had a bug

== DM01 firmware versions ==
Links to older and current firmware versions '''(strikethrough means not recommended)''':

'''May 29, 2024''' - [https://to7motor.com/wp-content/uploads/2024/05/TO7-DM01-K240529A-V6026C.zip K240529A-V6026C]

'''Jul 24, 2024''' - [https://to7motor.com/wp-content/uploads/2024/07/TO7-DM01-K240724A-V6306C1.zip K240724A-V6306C1] ([https://discord.com/channels/713249006129512448/1093198123821694976/1382842477626790051 regarded as the best version] by many)

'''Aug 02, 2024''' - [https://to7motor.com/wp-content/uploads/2024/08/TO7-DM01-K240802A-V6207C.zip K240802A-V6207C]

'''Oct 16, 2024''' - [https://to7motor.com/wp-content/uploads/2024/10/TO7-DM01-K241016-V6212C.pack%20.zip K241016-V6212C]

<s>'''Dec 16, 2024''' - [https://to7motor.com/wp-content/uploads/2024/12/TO7-DM01-K241216-V6400C.zip K241216-V6400C]</s>:
* Factory release notes [https://drive.google.com/file/d/13P2v15ppR2yadAu6koY15tS9TGCWH2iM/view?usp=drive_link here].
* Amps are at different values but are hard set and cannot be changed (see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* Speed is capped at 32kph throttle-only and 40kph while pedalling. (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* Some settings refuse to save. See [https://discord.com/channels/713249006129512448/1093198123821694976/1320528990561374262 this video].
* It is not possible to roll back to an older version after installing this one
<s>'''May 09, 2025''' - [https://to7motor.com/wp-content/uploads/2025/05/TO7-DM01-K250509-V6500-3.zip K250509-V6500]</s>:
* Fixes issue of settings not being saved on motors purchased before Oct 2024.
* Amps are still hard set and cannot be changed (see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* Speed is capped at 32kph throttle-only and 40kph while pedalling. These can still be changed if you have access to the advanced menu. (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* It is not possible to roll back to an older version after installing this one
* There are [https://discord.com/channels/713249006129512448/1093198123821694976/1385835905234763876 reports of this version bricking controllers].
'''Aug 1st, 2025''' - [https://to7motor.com/wp-content/uploads/2025/08/TO7-DM01-K250731-V6510C.zip K250731-V6510C]

:'''''Manufacturer's release notes''''':
* Resolved the controller malfunction that occurred after the system upgrade
* Enhanced the performance of the torque sensor for greater stability 
* Further improved overall riding quality for a smoother and more comfortable experience.
:'''''Other''''':
* Amps are still hard set and cannot be changed (see [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* Speed is capped at 32kph throttle-only and 40kph while pedalling. These can still be changed if you have access to the advanced menu. (See [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|Factory default settings]]).
* It is not possible to roll back to an older version after installing this one
* Jonas reports that this firmware is great so far but the default settings leave something to be desired. He has some suggestions to improve it. See [[Example Setups#Jonas’s DM01 settings|Jonas’s DM01 settings]].
'''Sep 26, 2025''' - [https://to7motor.com/wp-content/uploads/2025/09/TO7-DM01-K250926-V6510C-3.zip K250926-V6510C-3]

:'''''Manufacturer's release notes''''':
* Enhanced Throttle Response
* Accelerated Cadence-Based Startup
* Optimized Battery Level Indication
:'''''Other''''':
* Expected that this version also has the same speed and amp restrictions as other recent versions, and will not allow rollback to an older version

'''Nov 27, 2025''' - [https://to7motor.com/wp-content/uploads/2025/11/TO7-DM01-K251127-V6510C.zip K251127-V6510C]

:'''''Manufacturer's release notes''''':
This release focuses on making your controller even more reliable and efficient, with key improvements in:
* Smoother Operation: Better response speed and control precision.
* Stronger Performance Under Pressure: Enhanced stability and self-protection during sudden load changes.
''We would like to clarify that the "increased efficiency" mentioned in this update primarily refers to improvements in system control efficiency and more rational energy utilization, rather than an increase in the motor's own electro-mechanical conversion efficiency. So this updated FW does not increase the peak power or directly reduce motor heat generation under sustained high load. The true goals of our optimization are to enhance your riding experience and system reliability, which are reflected in:''

''- Smoother Response: Reduced abruptness in power delivery, resulting in more linear and controllable acceleration and deceleration.''

''- Enhanced Self-Protection: The controller can now more rapidly and smoothly adjust its output during sudden load changes, protecting the motor and battery from potential electrical stress.''

''In short, this update allows the existing hardware to perform in a more stable, smoother, and more reliable manner''

:'''''Other''''':
* Expected that this version also has the same speed and amp restrictions as other recent versions, and will not allow rollback to an older version

'''Jan 08, 2026''' - [https://to7motor.com/wp-content/uploads/2026/01/TO7-DM01-K260108-V6510C.zip K260108-V6510C]

* No release notes but it is expected that this version also has the same speed and amp restrictions as other recent versions, and will not allow rollback to an older version

<s>'''May 14, 2026''' - [https://to7motor.com/wp-content/uploads/2026/05/TO7-DM01-K260511-V6510C.zip K260511-V6510C]</s>

* This version is mainly about hardware protection and stability. Our engineers reworked the current management layers to make the controllers much more resistant to voltage spikes and high-load stress, which significantly minimizes the chance of component failure.

'''May 22, 2026''' - [https://to7motor.com/wp-content/uploads/2026/05/TO7-DM01-K260521A-V6510C.zip K260521A-V6510C]

* As above. It is assumed that since this version came out so soon that the previous version had a bug

== Older Display firmware versions ==
These versions should be fine to update from the old M1/M2 options, adding Eco/City/Sport modes, and should not remove access to the Advanced menu. If you have an older Display and are having trouble installing a more recent firmware version, try installing one of these first.

'''T24''' - Oct 2024 - [https://to7motor.com/wp-content/uploads/2024/10/TO7-T24V-Y241016.hex_.zip TO7-T24V-Y241016.hex_.zip]

'''T154''' - April 2024 - [https://to7motor.com/wp-content/uploads/2024/04/TO7-T154-Y240402A-Shut-down-M1M2.zip TO7-T154-Y240402A-Shut-down-M1M2.zip]

'''M15''' - April 2024 - [https://to7motor.com/wp-content/uploads/2024/04/TO7-M15-Y240430A.zip TO7-M15-Y240430A.zip] 

'''LCD-135C''' - June 2024 - [https://to7motor.com/wp-content/uploads/2024/06/TO7-LCD135C-Y240621A.zip TO7-LCD135C-Y240621A.zip] (The beta version [https://www.mediafire.com/file/hd3vjyfe5nvvzwr/LCTO7F-LCD135C-Y251024.hex/file LCTO7F-LCD135C-Y251024 available here] solves a number of issues including the "I" button not working correctly).

== Locked Displays ==
Updating the Display to [https://to7motor.com/progam-updating-instructions#display the latest firmware versions] released in Dec 2024 - Jan 2025 '''will''' lock you out of the Advanced menu, and it is unlikely to be reversible. (The dates can be seen in the URLs if you hover over the links. eg [https://to7motor.com/wp-content/uploads/2024/12/TO7-T24V-Y241225.zip https://to7motor.com/wp-content/uploads<span style="color:red">'''/2024/12/'''</span>TO7-T24V-Y241225.zip])

There is also an indication of this in some of the firmware filenames. e.g. T154 - [https://to7motor.com/wp-content/uploads/2025/01/TO7-T154-Y240927-No-advanced-menu.zip https://to7motor.com/wp-content/uploads/2025/01/TO7-T154-Y240927-<span style="color:red">'''No-advanced-menu'''</span>.zip]

Note that if you do install the latest version anyway, one anomaly, at least with the T24 Display, is that [https://discord.com/channels/713249006129512448/1093198123821694976/1346689533390880878 the version number does not reflect in the numbers on the Info screen]. The file name for the latest T24 version is TO7-T24V-Y<span style="color:red">'''241225'''</span>, yet the Display will show it as 24121.
[[File:Picture169.png|frame|none]]

ToSeven will advise how to unlock the Advanced settings on request if you have a reason to do so and understand that it will void warranty.

== Controller firmware update tool versions ==
In some cases, attempting to install or reinstall certain controller firmware versions, an older update tool will give “connect fail” or other errors, while the latest version will work, or vice versa. If you have trouble installing a particular firmware version, it may be worth trying a different version of the tool.

Dec 2023 [https://to7motor.com/wp-content/uploads/2023/12/FOC-IAP-Tool-Ver-3.1-CMD-119-20002.zip FOC-IAP-Tool-Ver-3.1-CMD-119-20002.zip]

Aug 2024 [https://to7motor.com/wp-content/uploads/2024/08/1_TO7_MiddleMotor_IAP_Tool_ForWin7-Win11.zip 1_TO7_MiddleMotor_IAP_Tool_ForWin7-Win11.zip] 

Dec 2024 [https://to7motor.com/wp-content/uploads/2024/12/TO7-G2-IAP-Tool-Ver-5.1-20002-2.rar.zip TO7-G2-IAP-Tool-Ver-5.1-20002-2.rar.zip]

Mar 2025 (Better English language support)  [https://to7motor.com/wp-content/uploads/2025/03/TO7-G2-IAP-Tool-Ver-5.2-CMD-20002-2025-3-27-%E8%8B%B1%E6%96%87%E5%B7%A5%E5%8E%82%E7%89%88.zip TO7-G2-IAP-Tool-Ver-5.2-CMD-20002-2025-3-27-英文工厂版.zip]

Instructions to use it are the same for all versions. See “[[#How to update the firmware|How to update the firmware]]” above.

== Display firmware update tool ==
Note that to update firmware on Displays, a different tool is required. This is available [https://to7motor.com/progam-updating-instructions here] along with instructions to use it, or it can be downloaded directly from the link below.
* [https://to7motor.com/wp-content/uploads/2023/12/MH2020C%20ISP.zip MH2020C_ISP.zip]

Note that because it is necessary to disconnect the Display from the battery to update firmware, it then needs to get power from somewhere else, which is what the included power adapter is for.

Be aware also that this procedure is a little tricky, as mentioned in "[[#How to update the firmware|How to update the firmware]]" above. Follow the instructions carefully. The power button must be held down during the process. Once completed, the “success” message is in Chinese. When it appears you have 10 seconds to disconnect the cable or the process will start again. Disconnecting the cable during the process may corrupt and brick the Display, but it will usually be recoverable by repeating the process.

== “Conntect Fail” (Connect Fail) error ==
This error is usually caused by a mismatch between the controller version and the update tool version. [[#Controller firmware update tool versions|See above]]. It can also occur if attempting to update to a newer version from a very old version. In these cases it may be necessary to [https://discord.com/channels/713249006129512448/1093198162417688656/1316564761730879489 try installing a version in between the two versions, then try again].
[[File:Picture170.png|frame|none]]

== “MCU unidentified, try again or IAP version is lower or pack version is lower” error ==
The error below will come up when attempting to roll back to an older firmware version after installing firmware after November 2024 for the DM02 or December 2024 for the DM01. There is no known solution for this so far.
[[File:Picture171.png|frame|none]]

== “Access violation at address 000000000” Error ==
Similar to above, this error may occur when attempting to program blocked or lock-out firmware. (Any firmware version that has the last digits as V6400 or higher will be a lockout version). Contact ToSeven for a solution.

== “Program Fail” error ==
ToSeven have identified some faulty programming cables which were shipped out between April and July 2025. Using one of these cables will result in a “Program Fail” error when attempting to update, followed by a “[[Troubleshooting#What does the “Communication Interrupted” error message mean|communication interrupted]]” error on the display when booting up the motor afterwards.This is recoverable using a properly functioning programming cable [https://discord.com/channels/713249006129512448/1093198162417688656/1406940639035133973 in most cases].
[[File:Picture172.png|frame|none]]
