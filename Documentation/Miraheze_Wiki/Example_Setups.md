# Example Setups

Source: https://toseven.miraheze.org/wiki/Example_Setups

The following are user recommendations and may be customised to suit individual preferences. For example, if less or more power is required in a particular level, adjust power, speed or amps accordingly. Keep in mind that more power & speed can mean more heat generation, faster battery consumption, increased wear & tear, and may void warranty.
Settings are based on a 48V battery on the 500W motor unless otherwise specified.

'''Note also that it is essential to [[General FAQs#Should you configure the motor's voltage setting|specify the correct battery voltage in the battery settings]]. Failure to do so may result in irreversible damage to the motor or poor performance.
'''

== The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware) ==
'''Controller''' (amps are hard set as follows):

{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:20%;" | 
! style="width:20%;" colspan="2" | DM02 (500W)*
! style="width:20%;" colspan="2" | DM01 (1000W)*
|-
! style="width:20%;" | 
! style="width:20%;" | 36V
! style="width:20%;" | 48V
! style="width:20%;" | 48V
! style="width:20%;" | 60V
|-
| '''Eco''' || 9 || 7.1 || 10 || 8.3
|-
| '''City''' || 12 || 9.5 || 18 || 15
|-
| '''Sport''' || 18 || 14.2 || 30 || 25
|}

<nowiki>*</nowiki> For other wattage motors, check the sticker or engraving on the bottom of the motor for maximum recommended amps.
For other voltage batteries for the DM02, the maximum amps can be calculated using the formula: <36v E/C/S value (from the 2nd column above)> × 38 ÷ <battery voltage>. eg 18 × 38 ÷ 48V = 14.2A (Fully charged 48V is 18 × 38 ÷ 54 = 12.66A)

For the DM01, the formula seems to be <48v E/C/S value (from the 4th column above)> × 50 ÷ <battery voltage>. eg 30 × 50 ÷ 60V = 25A.

Note that this also means that the amps go up as the battery discharges, providing more even power across the range.

A basic formula for peak power is: volts x controller amps = watts. eg a fully charged 48V battery on the 500W DM02 is 54V × 14.2 = 766 watts.

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:25%;" | Level
! style="width:25%;" | Spd
! style="width:25%;" | Pwr
! style="width:25%;" | EAssist
|-
| 1 || 10kph / 6mph* || 25% || 30%
|-
| 2 || 15kph / 9mph || 35% || 50%
|-
| 3 || 20kph / 13mph || 50% || 60%
|-
| 4 || 32kph / 20mph || 70% || 80%
|-
| 5 || 40kph / 25mph || 100% || 100%
|}

<nowiki>*</nowiki> DM01 Level 1 is 6kph/4mph and 20% power.

'''PAS > SpeedMode >''' Speed_MaxLevel

'''TorqueSensor > SpeedMode >''' Speed_MaxLevel
 
'''Throttle > SpeedMode >''' Level 7. 

See also [[Firmware#DM02 firmware versions|DM02 firmware version notes]] and [[Firmware#DM01 firmware versions|DM01 firmware version notes]]

== The Goody Two Shoes ==
Close to the factory settings in the latest firmware to stay safe but with a bit more speed in the lower levels, more progressive power levels, and a reduction in the EAssist levels for less aggressive torque sensor behaviour.

'''Controller''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:33%;" | 
! style="width:33%;" | 36V
! style="width:33%;" | 48V
|-
! Eco
| 9
| 8
|-
! City
| 12
| 12
|-
! Sport
| 18
| 16
|}

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:25%;" | Level
! style="width:25%;" | Spd
! style="width:25%;" | Pwr
! style="width:25%;" | EAssist
|-
| 1 || 15kph / 9mph || 20% || 10%
|-
| 2 || 20kph / 13mph || 40% || 20%
|-
| 3 || 25kph / 16mph || 60% || 40%
|-
| 4 || 32kph / 20mph || 80% || 60%
|-
| 5 || 40kph / 25mph || 100% || 80%
|}

'''PAS > SpeedMode.''' Set to “Speed_Level”. 

'''TorqueSensor > SpeedMode.''' Leave at Speed_MaxLevel or set to “Speed_Level” if you want it to match the speed of the level you are riding in.

'''Throttle > SpeedMode''' > Level 7.

== EN15194 compliant ==
EN15194 is a European standard for "Pedelecs" or electrically power-assisted cycles (EPACs). It specifies requirements for the safety, performance, and design of e-bikes with a continuous rated output of 250W and a maximum assisted speed of 25 km/h. Note that "continuous", also often referred to as nominal power, does not mean that the motor cannot provide more than that as "peak" performance for short periods of time before they are supposed to overheat. Click [http://General_FAQs#Why_is_there_more_than_500_watts_showing_on_the_display_while_riding_the_500W_DM02 here] for more.

The settings below, if applied to a motor rated at higher than 250W will provide extra torque and power while giving '''the appearance''' of being street legal if stopped and subjected to basic tests. (To be actually legal in most countries, the motor must be factory rated and locked as 250W/25kmh. Motors than can be easily unlocked with a button or by pressing a sequence on the display are generally still not strictly legal). However, while behaving like a legal motor, you may still get in trouble if you are pulled over by a dedicated task force who knows what they are looking at, or you have been observed speeding or hooning, or if you are involved in an accident, which can often mean the bike will be forensically tested. Note that while a [https://vi.aliexpress.com/item/1005008567243745.html fake EN15194 sticker] can help with a standard roadside stop by a vaguely suspicious officer, this is becoming an additional offence in some countries now, and it also implies intentional deception (i.e. you knew very well that it was illegal). So it may be better to not have one and just claim that it is compliant as far as you know.

'''Main settings:'''

'''Device > SpeedSensorFail''' set to ‘Speed Unlimit’. This can help reduce the dragging sensation when the speed cutoff is reached.

'''Advanced Settings:'''

'''Controller''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:33%;" | 
! style="width:33%;" | 36V
! style="width:33%;" | 48V
|-
! Eco
| 9
| 7.1
|-
! City
| 12
| 9.5
|-
! Sport
| 18
| 14.2
|}

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:25%;" | Level
! style="width:25%;" | Spd
! style="width:25%;" | Pwr
! style="width:25%;" | EAssist
|-
| 1 || 18kph / 11mph || 25% || 20%
|-
| 2 || 25kph / 16mph || 40% || 40%
|-
| 3 || 25kph / 16mph || 60% || 60%
|-
| 4 || 25kph / 16mph || 80% || 80%
|-
| 5 || 25kph / 16mph || 100% || 100%
|}

'''PAS > SpeedMode'''. Set to “Speed_Level”. (See “[[#The Hauler|The Hauler]]” for why).

'''TorqueSensor > SpeedMode.''' Set to “Speed_Level”. (‘NoSpeedLimit’ will ignore the speed cutoff.)

'''Throttle > SpeedMode'''. Set to level 3 (6kph)

== The Hauler ==
For heavier riders or carrying loads. These settings work equally well for me riding the DM02 in PAS mode or with the torque sensor enabled. The Spd & Pwr values are mostly for PAS riding while the Eassist values are for torque sensor enabled riding. With the torque sensor enabled, I spend most of my time in City mode, levels 2-4. I only use the higher power and speed levels for short bursts. Sustained use may increase heat and wear & tear.

'''Controller'''
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:33%;" | 
! style="width:33%;" | 36V
! style="width:33%;" | 48V
|-
! Eco
| 12
| 10
|-
! City
| 16
| 14
|-
! Sport
| 20
| 18
|}

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:800px;"
|-
! style="width:20%;" | Level
! style="width:20%;" | Use
! style="width:20%;" | Spd
! style="width:20%;" | Pwr
! style="width:20%;" | EAssist
|-
| 1 || Shared Paths || 18kph / 11mph || 25% || 20%
|-
| 2 || Bike paths || 25kph / 16mph || 40% || 40%
|-
| 3 || Commuting || 30kph / 19mph || 60% || 60%
|-
| 4 || Fast Cruising || 35kph / 22mph || 80% || 80%
|-
| 5 || Off road || 40kph / 25mph || 100% || 100%
|}

'''PAS > SpeedMode'''. Set to “Speed_Level”. This will make the motor cut power when the speed is reached in each level, making it easier to maintain a set speed in PAS mode. There is a sweet spot here (which may take a little experimenting to find depending on your weight and your local terrain) where the motor will have just enough power to reach the speed but no more. Too much power can have a yoyo effect, launching the bike beyond the speed level and then cutting power over and over. See also “[[Troubleshooting#Dragging sensation when speed limit is exceeded|Dragging sensation when speed limit is exceeded]]” which may help with this as well.
Be aware also that it can also take several pedal rotations before the motor kicks in again after the speed cutoff has been exceeded.

'''TorqueSensor > SpeedMode'''. Set to Speed_MaxLevel or NoSpeedLimit. 

'''Throttle > SpeedMode'''. Set to level 12 Scooter mode.

== One Mode to Rule Them All ==
Ian has been experimenting with using 8 levels and only using Sport mode, but Eco and City modes are still there in case needed.
'''
Application > LevelCount''' - set to 8 (Note, if changing from 5 to 8 levels you will need to scan through and check all of the levels as it can leave some peculiar default values).

'''Controller'''
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:33%;" | 
! style="width:33%;" | 36V
! style="width:33%;" | 48V
|-
! Eco
| 9
| 8
|-
! City
| 14
| 12
|-
! Sport
| 18
| 15
|}

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:25%;" | Level
! style="width:25%;" | Spd
! style="width:25%;" | Pwr
! style="width:25%;" | EAssist
|-
| 1 || 15kph / 9mph || 11% || 6%
|-
| 2 || 25kph / 16mph || 21% || 14%
|-
| 3 || 30kph / 19mph || 30% || 20%
|-
| 4 || 35kph / 22mph || 40% || 30%
|-
| 5 || 35kph / 22mph || 50% || 40%
|-
| 6 || 35kph / 22mph || 60% || 50%
|-
| 7 || 40kph / 25mph || 80% || 60%
|-
| 8 || 50kph / 32mph || 100% || 100%
|}

== Jonas’s DM01 settings ==
With the [[Firmware#DM01 firmware versions|Aug 1st, 2025 firmware release]], Jonas reports that it works well but the default settings are not the greatest: 
:“''there's a weird interdependency of eassist and power. I have to keep power pretty high and eassist rather low, way better torque response than stock but I'm still fiddling around with values. If you go higher you hit max power already at level 2 or 3, so you lose dynamics. Eeassist has always been a mystery box, and now it behaves differently.''”

'''Speed & Power''' 
{| class="wikitable" style="text-align:center; width:500px;"
|-
! style="width:25%;" | Level
! style="width:25%;" | Spd
! style="width:25%;" | Pwr
! style="width:25%;" | EAssist
|-
| 1 || 45kph / 28mph || 46% || 20%
|-
| 2 || 45kph / 28mph || 54% || 32%
|-
| 3 || 45kph / 28mph || 62% || 44%
|-
| 4 || 45kph / 28mph || 70% || 56%
|-
| 5 || 60kph / 38mph || 82% || 68%
|}

Device > Shift Sensor > mode > pwr 10%

PAS > Keep Time > 200ms

Torque Sensor > mode > sensitive

Torque Sensor > keep time > 200ms

Torque Sensor > startup voltage > 300mV
