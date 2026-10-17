# Troubleshooting

Source: https://toseven.miraheze.org/wiki/Troubleshooting

== What does the flashing yellow triangle on the display mean ==
On the T24 display long press the Info (I) button while the triangle is flashing to find out what it means. For the T154 display, short press the power button until “Info” is displayed at the bottom of the screen, then press the down arrow. There are a variety of error messages and solutions explained in the manual:
[[File:Picture10.png|800px|frameless|center]]
[[File:Picture11.png|800px|frameless|center]]
[[File:Picture12.png|800px|frameless|center]]

== Basic troubleshooting steps ==
# See if the problem is mentioned in the chart above or in the FAQs below. For mechanical issues such as the motor slipping or chain skipping see the [[Installation_and_bike_fit|Installation and bike fit]] and [[Chains, chainrings and chain lines]] FAQs
# Check all cables are connected properly, particularly [[#Brick_stories:_Corrupted_controller_from_partially_seated_cable|the main motor cable]]. This requires some muscle to get all the way in and is a frequent cause of problems. 
# Check that [[#Display_turns_itself_off_just_after_it_starts|the speed sensor]] is installed correctly
# Check [[#System_boots_up_but_no_power_from_motor|the brake sensors are not tripped]]
# If nothing powers up, try another battery if possible, or try your battery on another motor. This will narrow down whether it is a problem with the battery or with the motor.
# [[Firmware|Install the latest firmware]] for both the controller and the Display
# Ask in the [https://highvoltagekits.com/discord High Voltage Discord DM01 and DM02 channels] 
# Contact the manufacturer

== What does the “Motor Disconnect” error message mean ==
[[File:Picture160.png|frame|none]]
This indicates that the display cannot see the controller. In most cases this would be a connector not plugged in correctly, such as [[#Brick stories: Corrupted controller from partially seated cable|the main cable not fully seated]]. Also check for bent pins. Failing that, it could be a faulty cable or faulty controller. In [https://discord.com/channels/713249006129512448/1370982874366480515/1371147908682092617 the case discussed here], a loose screw on the controller was found to be the issue.

== What does the “Motor Blocked” error message mean == 
[[File:Picture161.png|frame|none]]

While “Motor blocked” can be a symptom of a [[#What is a bricked controller|bricked controller]] if it won’t go away again, there are also some other conditions that can trigger it temporarily. Kat from ToSeven had this to say via WhatsApp.

:''Recently, we received feedback from some customers reporting a "motor blocked" issue. We would like to draw your attention to the following main reasons why you may see this error:''

:# ''If the user's riding load is too heavy or the road conditions are poor, it can cause the motor to reach its maximum torque, triggering the protection mechanism to stop the motor temporarily.''
:# ''When the user's bicycle experiences an impact load or the motor stalls while climbing a slope, the two phases remain connected for an extended period, and inductor saturation can also lead to overcurrent.''
:# ''Sudden acceleration or braking of the e-bike can cause large body inertia. If the acceleration/deceleration time is set too short, the synchronous motor speed changes too rapidly, causing the rotor and stator magnetic field to lose synchronization. This can result in an induced electromotive force that is too large, leading to overcurrent.''

:''Our DM mid-motor has a protection setting in place to ensure that the power supply can be cut off promptly when the motor is overloaded, preventing motor damage. This is why some users may see the "Motor blocked" message on their displays.''

:''We kindly recommend all users ensure the load is not too heavy when riding with our DM mid-motor, and avoid excessive use of the motor on steep or uneven terrain to reduce the risk of the motor reaching its maximum torque.''

'''Note''' that this has also been addressed in more [[Firmware#DM02 firmware versions|recent firmware]] so update firmware if you witness this fault.
 
A faulty torque sensor can cause this error to appear, usually while climbing a hill or just afterwards. If the torque sensor becomes stuck at a higher voltage, it can cause it to max out and trigger the motor’s protection mechanism. See “[[#How to test the torque sensor |How to test the torque sensor]]” below.

== What does the “Communication Interrupted” error message mean ==
[[File:Picture162.png|frame|none]]
Check [[#Brick stories: Corrupted controller from partially seated cable|the main motor cable is correctly seated]]. Failing that, this is usually a sign of a bricked controller but may possibly be recoverable with a firmware reflash. See also “[[Firmware#“Program Fail” error|Program Fail]]” error.

== What does the “Controller Broken” error message mean ==
[[File:Picture163.png|frame|none]]
Check all connectors and try reinstalling firmware but this usually means the controller has failed and will need to be replaced.

== System boots up but no power from motor ==
Check that a brake sensor isn't in a tripped state. This can happen if either a magnet or sensor has become displaced or if they are too far apart. Occasionally this can also be due to a faulty sensor. On the T24 display, the icon below will appear if a sensor has been tripped.
[[File:Picture69.png|frame|none]]
For other displays without an icon, or for troubleshooting in general, first unplug one sensor to see if the problem goes away, then the other. This will identify which one is tripping so it can be checked and replaced as necessary.

Another possible cause of this issue is if the power level is set to level zero.

== Display goes crazy and/or no power ==
This is usually because the speed sensor is not connected properly or is too far from the magnet on the spokes. You will also usually see a yellow triangle flashing within a few seconds when you start riding, then a sudden loss of power. Also check that the level selected isn’t 0, and the main cable is plugged in all the way. The main connector can be tough and may require some muscle to get all the way in. See “[[#Brick stories: Corrupted controller from partially seated cable|Brick stories]]” below.

== Display turns itself off just after it starts ==
This is most likely due to a communication problem between the display and the controller. If the display can’t see the controller, it shuts down again. Check the main cable connector. This one is tight and requires some muscle to get all the way in.

Another common cause is, like the previous FAQ, the speed sensor has not been installed correctly. The spoke magnet should be within 5 - 15mm of the sensor. Anything outside of that may not be being read so the display assumes nothing is happening and automatically shuts down after 3 minutes of no activity. Watch the LED on the sensor. If it doesn’t light up when the magnet passes, it’s not being read.
[[File:Picture70.png|frame|none]]

Also, be sure the spoke magnet lines up with the centre of the sensor, not the round part.
[[File:Picture71.png|frame|none]]

== System shuts down under load ==
A possible explanation for this is that the battery does not have enough current for the controller's needs. This can trigger the battery's BMS and make it shut down. Over time, this can damage the controller. See "[[Batteries#What to look for in a battery|What to look for in a battery]]".

== Display goes to 99 kph just after starting and power cuts out ==
The sensor is too close to the magnet as above.

== Display is completely black or difficult to see in daylight when wearing sunglasses ==
Polarized sunglasses will black out TFT/LCD screens. You can test if your sunglasses are polarized by holding them in your hand while looking through them at the display, then rotate them 90°. At some points you may be able to see the display clearer than others. The only solution is to buy some non-polarized sunglasses for cycling. More info on this [https://www.allaboutvision.com/sunglasses/polarized-lenses-and-lcd-screens/ here].

== Display randomly resets ==
Currently under investigation but this has happened to several people. One cause might be a button sequence, something like [https://discord.com/channels/713249006129512448/1093198162417688656/1333207576434704533 pressing the on button to boot up but not long enough to actually turn on and then pressing the on button again within 5 seconds]. Others have reported it happening after resetting the timer by holding the power and down button together, which is the correct way to do this but it shouldn’t reset everything, just the timer and averages. This might be perhaps holding these buttons down for too long but it may also be a bug.

== Buttons not working correctly on LCD135C Display ==
If you find the buttons not working correctly on the LCD135C, such as the "I" button scrolling down instead of working as it should, or some buttons need to be pressed multiple times to get a response, first update the firmware to the [https://www.mediafire.com/file/hd3vjyfe5nvvzwr/LCTO7F-LCD135C-Y251024.hex/file beta available here].  Note that this version was released 25/10/2025. There is now a newer version on [https://to7motor.com/progam-updating-instructions#display ToSeven's website here], so try that first.

If upgrading firmware doesn't fix it, the buttons can be calibrated as demonstrated in [https://youtu.be/h8ZY8BGZ8Ns this video].
<embedvideo service="youtube" width="640" alignment="">h8ZY8BGZ8Ns</embedvideo>

== Average Speed automatically resets at 119.3km with T154 Display ==
A known bug demonstrated [https://discord.com/channels/713249006129512448/1093198162417688656/1330102603106746409 here].
ToSeven are aware of this issue and should have [https://to7motor.com/progam-updating-instructions#display an updated firmware version] for it in due course.

== No power with torque sensor enabled ==
If you get no power from the motor when the torque sensor is enabled, but the throttle and PAS mode (torque sensor disabled) still work fine, (as seen in [https://youtu.be/A40O-YdPiNg?si=JvBjJeTpgR10fUaV this video]) then this is most likely a faulty controller. However, it is also possible that similar symptoms could occur from a faulty torque sensor. If it passes [[#How to test the torque sensor |the torque sensor test]] then it is more likely a faulty controller.

== What is a bricked controller == 
If a display becomes unresponsive, then the controller is referred to as “bricked”, meaning it’s then only good for being a paperweight or brick. This can sometimes happen while [https://discord.com/channels/713249006129512448/1093198123821694976/1348986809421664400 attempting to upgrade firmware], or sometimes inexplicably after changing settings. Chances are there was a fault with the controller all along and this was going to happen to it eventually anyway. It can also happen if mistreated, such as exposure to water or a sudden temperature change such as hitting a puddle with a hot motor, or using settings in the [[Display Settings#Advanced Settings|advanced menu]] that are outside of its safe limits. Unfortunately, there is no recovery procedure, and the controller will need to be replaced.

Common symptoms are a “[[#What does the “Motor Blocked” error message mean |Motor Blocked]]” message that won’t go away, or “[[#What does the “Communication Interrupted” error message mean|Communication interrupted]]”. Another common symptom is an unusual amount of resistance when pushing the bike backwards.

== Brick stories: Corrupted controller from partially seated cable ==
A user reports their motor and display became unresponsive after attempting to navigate the menus after recently installing the motor. It was later discovered that the motor to display cable connection was not fully seated. Resolution of this required use of a programming cable to reinstall firmware (in this user’s case, an older [[Firmware#Controller firmware update tool versions|IAP programming tool]] and firmware version was needed to successfully reinstall).
 
'''Lesson: ENSURE CABLES ARE FULLY SEATED.'''

{| class="wikitable"
|-
! <span style="color:red">NOT FULLY SEATED – BAD</span>
! <span style="color:green">FULLY SEATED – GOOD</span>
|-
| [[File:Picture164b.png|frame|none]]
| [[File:Picture165.png|frame|none]]
|}

'''Doug also reports the same thing happening to him.'''
:''I can confirm an incorrectly seated cable can give comms error ([[#What does the “Communication Interrupted” error message mean|Communication Interrupted]]) and corrupt firmware. Happened to me,  I thought controller was dead but firmware update corrected the issue so you can add a second report of incorrectly seated cable causing firmware corruption.''

== Most or all power is suddenly lost when pedaling hard up a hill or after a gear change ==
This can be a faulty torque sensor or “[[#What does the “Motor Blocked” error message mean |motor blocked]]” error due to overload. It can also happen if the controller overheats. See “[[#How to test the torque sensor|How to test the torque sensor]]” below and the [[Display Settings#TempSensor|Temperature Settings]].

== Ghost pedalling or gradual loss of power with torque sensor enabled (DM02) ==
While less likely to break than [https://endless-sphere.com/sphere/threads/tsdz2-torque-sensor-broken.98665/ the Tongsheng design], the ToSeven torque sensor does have its own issues. One is that it is a cheap electronic component which can sometimes fail or have a manufacturing defect. Another is that it measures strain across metal components ([https://en.wikipedia.org/wiki/Wheatstone%20bridge wheatstone bridge]) which also expand and contract with heat, which can throw its calibration out during a ride. The voltage range measured is 0.6 - 0.7V at rest, up to 3.3V when maximum pressure is applied to the pedals. As the voltage goes up with more pressure, more power is delivered from the motor, giving the effect of multiplying the torque from your legs.This works great when the motor is cool. However, as the motor gets hot, the resting voltage for the torque sensor rises as well, so instead of 0.6 - 0.7V at rest, you may see anything up to 1.2V or more. The motor then thinks that there is pressure on the pedals when there is not, which can manifest as some slight ghost pedalling, or behaving like it has gone up a few power levels, or the motor has somehow switched itself to PAS mode. For most people riding on fairly flat surfaces, this voltage increase may be barely noticeable, or perhaps even a positive thing since the bike will feel easier to ride as it happens. However, this is the response under light loads. Under heavier loads such as climbing hills [https://discord.com/channels/713249006129512448/1093198162417688656/1386283799872016465 there may be noticeably less power available] and/or very little torque sensor response. 

'''Solutions''': A reboot will recalibrate the torque sensor to the increased voltage/temperature. However, while this will feel more normal again and should get rid of the ghost pedalling, it may not feel quite as lively as when the motor is cool. The reason is possibly due to the voltage range for the torque sensor then being reduced. It may also be necessary to go up a power level to get the same "feel". If rebooting doesn’t get it behaving normally again and/or the issue quickly returns, then the torque sensor is possibly faulty. See also “[[#Torque sensor voltage changes when hitting bumps or bouncing the bike (Torque Sensor Voltage Deviance)|Torque Sensor Voltage Deviance]]”.

This behaviour has been improved in [[Firmware#DM02 firmware versions|more recent firmware since November 2024 for the DM02]], so update firmware if it becomes too annoying. The torque sensor resting voltage may still rise with heat, but it behaves much better if/when it does and may no longer be noticeable. However, it may still be present if riding at higher power levels which generate more heat. If riding at lower power is not acceptable, using PAS mode (disabling the torque sensor) may be the better option.

Note that the DM01 does not have this issue despite being equipped with an identical torque sensor. This is probably due to it having an externally mounted controller rather than internal, and/or its larger casing, giving it much better thermal management properties.

== Dragging sensation when speed limit is exceeded ==
The motor is designed to stop providing power when the speed is exceeded in the current level, which is useful to meet legal requirements in most countries as well as the ability to keep pedalling under your own power when assistance is no longer needed. (These limits can be customised in the Advanced settings under '''[[Display Settings#Speed&Power|Speed&Power]] > Speed'''. The way ‘speed’ is applied is also governed by '''PAS > SpeedLevel''', '''TorqueSensor > SpeedLevel''' and '''Throttle > SpeedLevel'''). However, some people report [https://discord.com/channels/713249006129512448/1093198162417688656/1373278829203816621 a dragging sensation when the motor cuts out], which is greatly improved if the '''SpeedSensorFail''' setting under [[Display Settings#Device|Device]] in the Main menu is set to ‘Speed Unlimit’ instead of ‘Speed Limit’.

== Re-engagement lag ==
One of ToSeven's biggest bugbears is re-engagement lag. This occurs after coasting where it can take several pedal strokes before the motor provides power again. [https://youtu.be/1k0xjmnOhnw?si=yCIb3os7Z3r5-PFS Example]. It is particularly noticeable after coasting downhill above a set speed cut off. As the bike drops below the cut off again to tackle the following hill, suddenly there is no power for what feels like a dozen or more pedal strokes. This means losing momentum and having to change down more gears than should have been necessary. It may also be noticeable after changing down gears while going up hill, even with no shift sensor attached, seemingly reacting to a sudden change in cadence. This one or two pedal strokes with little to no power also contributes to losing momentum faster than expected and having to change down more gears before the motor kicks back in and allows you to change up again for the remainder of the hill.

While changing Torque Sensor mode to Sensitive in the Advanced Settings can help it feel more snappy, there is no other known workaround for this issue at this time. Although, some people report that giving the throttle (if fitted) a quick blip or backpedalling slightly can help to wake the motor up faster. ToSeven are aware of it and can hopefully improve on it in future firmware updates.

== Power cuts in and out under load ==
If you experience a surging sensation, or it feels like the motor is losing power for a split second every now and then while under moderate to heavy loads, this can often be improved in the [[Display Settings#TorqueSensor|Advanced settings by setting Torque Sensor keeptime]] to 4000ms (40x100ms) or higher. The maximum it will allow is 5000ms (50x100ms). If you can't get rid of it, upgrading the firmware to the latest version will usually fix it. I experienced this with the DM01 running its original Nov 2024 firmware. Installing the September 2025 version solved it.

== Torque sensor voltage changes when hitting bumps or bouncing the bike (Torque Sensor Voltage Deviance) ==
This is most likely a faulty torque sensor. See “[[#How to test the torque sensor |How to test the torque sensor]]” below.
For more detail on this issue, see [https://discord.com/channels/713249006129512448/1296375346941267968/1296375346941267968 this thread] in the [https://highvoltagekits.com/discord High Voltage Light Electric Vehicles Discord channel].

== Low or no power after starting, requiring a restart to get it working properly ==
This is most likely a faulty torque sensor. See “[[#How to test the torque sensor |How to test the torque sensor]]” below.

== Timeout after torque sensor calibration ==
[[File:Picture166.png|frame|none]]

If you get a “Timeout” for the torque sensor calibration instead of “success” when booting up, you most likely have a faulty torque sensor. See “[[#How to test the torque sensor |How to test the torque sensor]]” below.

== How to test the torque sensor == 
To test the torque sensor:
* Start the motor as normal, then hit the down arrow to switch to level 0.
* Short press the power button repeatedly until “Test” appears at the bottom of the screen, then hit the ^ button 
* Short press the power button repeatedly again until “Drive” appears at the top.
[[File:Picture167.png|frame|none]]
* The torque sensor should show 0.6V to 0.7V when cool. Anything else is a problem.
* Now (assuming the voltage is within the correct range) hold both brakes and push on a pedal as hard as you can. If you’re strong enough, it may reach as high as 3.3V. How high it goes is not the issue but rather what happens when you take your foot off the pedal again. It should go back to 0.6V to 0.7V. Do the test several times. If it gets stuck at a higher voltage, you probably have a faulty torque sensor, and it will need to be replaced.
* If the resting voltage doesn't change from 0.6 - 0.7V no matter how much pressure is applied, try disabling the torque sensor then go for a ride to get the motor nice and warm, then re-enable the torque sensor and see if it has come good. More detail [https://discord.com/channels/713249006129512448/1093198123821694976/1452926000483663932 here]. 
* If it passes the test when cold but you experience unusual behaviour while riding such as [[#Ghost pedalling or gradual loss of power with torque sensor enabled (DM02)|ghost pedalling or feeling like the motor has switched itself to PAS mode or there is less power available]], update the firmware as this issue has been addressed in [[Firmware#DM02 firmware versions|more recent firmware versions]]. 
* Hit the ^ button to exit the test menu again.

'''Solutions?'''
First, [[Firmware#How to update the firmware|update the firmware]] to the most recent version as there are improvements to the torque sensor behaviour which overcome most of the reported problems. If you do notice that it’s not behaving quite right during a ride, then rebooting the system will also recalibrate the torque sensor to the increased operating temperature which should restore normal function. However, if the problem remains after rebooting, or it quickly returns, or you see a [[#Timeout after torque sensor calibration|calibration timeout]] * during booting, and/or it fails the test above, then the torque sensor is most likely faulty and will need to be replaced. See “[[Repairs and maintenance#DM02 torque sensor replacement|DM02 torque sensor replacement]]” above. 

<nowiki>*</nowiki> Note: If you do see a calibration timeout, then leaving it for a few days or tapping on the drive spindle (where the cranks bolt on) with a mallet may restore it, but the problem will frequently return and it will need to be replaced. Tap hard enough to cause a jolt but not hard enough to cause damage. In my experience, tapping on the right-hand spindle caused it to remain stuck but at various different voltages. Tapping on the left-hand spindle caused the voltage to drop until it reached 0. At this stage, performing the test above revealed no voltage increase when pressure was applied to the pedals. However, when rebooted, the voltage had returned to normal and it was functioning correctly again, at least until the next hill I rode up. 

For more detail on this issue, see [https://discord.com/channels/713249006129512448/1296375346941267968/1296375346941267968 this thread] in the [https://highvoltagekits.com/discord High Voltage Light Electric Vehicles Discord channel].

== How to test the throttle == 
As above:
* Short press the power button repeatedly until “Test” appears at the bottom of the screen, then hit the ^ button 
* Short press the power button repeatedly again until “Drive” appears at the top. The throttle should show 0.7V.
[[File:Picture167.png|frame|none]]
* Now hold one or both brakes (assuming the brake sensors are connected and functioning. Otherwise lift the back wheel off the ground) and hit the throttle. It should rise up to 3.3V, then return back to 0.7V when you let go. Anything outside of this could be a problem.

== How to test PAS is working == 
As above:
* Short press the power button repeatedly until “Test” appears at the bottom of the screen, then hit the ^ button 
* Short press the power button repeatedly again until “Drive” appears at the top. PAS speed should show 0 rpm.
[[File:Picture167.png|frame|none]]
* With the rear wheel off the ground, turn one of the cranks. It should show a change in the rpm. If you pedal backwards, it should return to 0 rpm.

== Headlight flashes or strobes when brakes engaged ==
If using [[Accessories#What lights are compatible|ToSeven lights]] connected using the supplied Y cable with red connectors, the longer cable (1200mm) is for the tail light. The shorter one (900mm) is for the headlight. (see [[Installation and bike fit#Where do all the wires go|wiring diagrams]] for more). The tail light cable allows the tail light to function as a brake light. If the headlight is connected to the tail light cable, it will behave like a brake light and react to the brakes being applied. This can mean being unusually dim and getting brighter when the brakes are applied, or flashing/strobing.

See also [[Display Settings#Device|Display Settings > Device]] for options for different brake light behaviour.
