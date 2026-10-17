# General FAQs

Source: https://toseven.miraheze.org/wiki/General_FAQs

== Are ToSeven mid drive motors worth buying ==

While this wiki has a lot of focus on what to do when things do go wrong, as well as adding detail for some aspects that have not been explained well by the manufacturer, this should not detract from the vast majority of customers who never have a problem and ride off happily into the sunset with their new motors. ToSeven motors represent great value for money, with performance that will put a smile on most faces, and are well placed to be the go-to DIY ebike kit in the near future. They have an improved design over Tongsheng motors and more configuration options. (More recent motors without access to the [[Display Settings#Advanced Settings|advanced settings]] can still be unlocked, with the understanding that warranty will be reduced or void in doing so, and the motor's service life may be reduced if pushed harder than it was designed for). Bafang BBS mid drives lack a torque sensor, and are [https://youtu.be/OiVlxlKsTjs?si=4_ogNdaQ7xN7tRJH now only being produced with CANbus controllers instead of UART] which will make them more difficult and more expensive to configure. CYC motors, while generally regarded as exceptional, are considerably more expensive than similar ToSeven motors.
While some manufacturing defects will occur, which is true for all manufacturers, there is support and guides from ToSeven for repairing them, and they are not overly difficult to repair.

== What is the difference between torque sensing and PAS ==
The torque sensor has the effect of multiplying your leg power, so you feel like you’re a lot stronger than you are; while still making you work a bit for it so you’re getting some exercise and maintaining some “feel” with the pedals. Cadence sensing, aka PAS (Pedal Assistance System), motors can only sense if the pedals are turning, providing power on that basis alone. This makes it easy to “ghost pedal”, meaning you get very little exercise. It can also make riding tricky terrain at low speeds a little challenging since the motor will want to lurch forward with more power than you might like whenever the pedals are turning. With that being said, cadence sensing is not necessarily a bad thing. For example, it’s great for commuting when you don’t really want to sweat, or if you have an injury or disability that prevents putting too much effort in. ToSeven mid-drive motors give you that option. Simply [[Display Settings#Device|disable the torque sensor in the settings]] and you have a cadence sensing (PAS) motor. Cadence sensing will also provide more power the faster you pedal, which gives a similar sensation to torque sensing.

== Mid drives vs hub drives ==
Hub drives have many advantages over a lot of mid drives. They are cheap and reliable, and don’t put stress on chains and cassettes, and can even still be ridden without a chain. This also means that cheaper 7 or 8 speed group sets will suffice, and are usually the most you can fit between the dropouts anyway on a rear hub. Where they mostly fall over is that they only have one speed and can’t use the bike’s gears, so they struggle with long or steep hills, and can overheat in these conditions. They add unsprung weight to the rear or front of the bike, making them less suitable for off road riding. Also, anything above 350 watts or so will need a torque arm fitted to prevent the motor from breaking free and [https://www.reddit.com/r/ebikes/comments/13zr27l/fork_broke_and_catapulted_my_face_into_the/ damaging the frame] and [https://www.reddit.com/r/ebikes/comments/1915vyo/ripped_wiring_on_hub_motorcan_i_diy_fix/ itself]. This torque arm also makes changing flats somewhat tiresome. It is also difficult to ride slow with them since they have a tendency to want to accelerate to the maximum speed they can manage in the level you are riding in, so you tend to do a lot of coasting and ghost pedalling. They can also chew a little more battery than a similarly powered mid drive.
For commuting, a hub motor may still be the best choice where you just want reliability and don’t want to sweat. However, a DM02 is usually priced similarly to a reasonable quality hub motor and doesn’t have the above downsides. It is easier to maintain a lower or higher speed, and you can still get a reasonable workout depending on the level you choose to ride in, and it will still feel like a bicycle rather than a scooter. For off road use, or for those who want higher performance, the DM01 is the better choice.

== Is the DM02 the same as the Tongsheng TSDZ2 ==
These two motors are similar to look at, but they are not the same. The engineer who designed the TSDZ2 left the company and started ToSeven. There are many design improvements such as improved temperature control, which was a common issue with the TSDZ2. See [https://youtu.be/E18k-oRYp_4?si=RELbTP6l3EhHIt3D this video] comparing the two motors. Another massive change is the ability of owners [[Display Settings#Advanced Settings|to alter advanced motor parameters]] directly from the display.

== How to choose which DM02 version to get (250, 350 or 500-watt) ==
If you want to be street legal, check your country’s ebike laws and choose accordingly. If you don’t care, go for the 500-watt version. It is easy to scale down the power in the settings if you find you have too much, but it is more difficult to soup up an electric motor if you want more power.

'''Note''': Check your country’s conditions. In some countries like Australia, riding an ebike that exceeds the laws can land you in trouble for riding an unregistered and unroadworthy moped, which can mean an expensive fine if you get caught, and they can and will confiscate your bike. With that being said, you are only likely to be stopped by police if you are doing something stupid like riding at excessive speed amongst pedestrians or disobeying road rules. Other countries can be more heavy-handed, with random roadside stops and checkpoints with officers armed with knowledge and equipment to check power levels, and they may even [https://x.com/LancsRoadPolice/status/1895165930134478978 seize the bike].
Be aware that if you are involved in an accident that causes someone’s injury or death while riding an illegal ebike, you could end up in jail. You will also have no insurance if you cause damage to someone or their property.

== What is the difference between the 250, 350 and 500-watt DM02 ==
Externally, all three versions are identical in size, and they all put out the same torque (up to 90nm) and the same peak watts (given the same controller amps and battery voltage). The main performance difference is that the lower wattage motors will taper off sooner so they take more work to reach and maintain higher speeds, and they may also get hotter quicker when pushed hard. From a standing start up a hill they all feel much the same.

Below is 500W version on left. 250W version on right. Identical length, width and height.
[[File:Picture200.jpg|frame|center]]

'''The 250W version''' is locked down to comply with legal requirements in some countries for a maximum of 250 watts continuous and is stamped with EN15194 compliance. It also comes with a 1T3 cable instead of 1T4, and does not come with a throttle or programming cable.
[[File:Picture1.png|frame|center]]
Note that the second part of that message indicates that it can be “hacked” with a programming cable and installing other firmware, while also warning that doing so may make the bike illegal according to your country’s laws. Be aware also that you may not be able to revert to the street legal version again once you’ve done it, which would defeat the purpose of buying the street legal version in the first place. May as well buy the 350 or 500-watt version, leave the throttle off, and slap a 250-watt sticker on it.

'''<span style="color:red">Warning:</span>''' It comes pre-installed with default settings for the global market, which means up to 40km/h per [[https://toseven.miraheze.org/wiki/Example_Setups#The_Factory_defaults_(From_Nov_2024_DM02/Dec_2024_DM01_firmware)|"the factory default" example here]]. A factory reset will also install these settings. To limit it to 25km/h for compliance in countries that only allow that will require access to the [https://toseven.miraheze.org/wiki/Display_Settings#Speed&Power Speed & Power settings in the Advanced menu], which requires a special key sequence which is available on request. ToSeven are working on country-specific firmware versions with a 25km/h restriction in the future.

'''The 500-watt version''' is a little different on the inside. Its rotor and motor windings are larger than the 250-watt and 350-watt versions, making it ~0.4kg heavier and more able to handle higher power. See image comparing the rotors from both motors below.
[[File:Picture2.png|frame|center]]

== Is there a difference between the 750W and 1000W DM01 ==
Yes. The [https://discord.com/channels/713249006129512448/1093198123821694976/1304841047192375393 1000W version is larger] to allow for a larger rotor & stator to cope better with the increased power. Note also that [https://www.aliexpress.com/item/1005007826717311.html different controllers are required for use with different battery voltages].
{| class="wikitable"
| [[File:Picture3.png|none]] 
| [[File:Picture4.png|none]]
|}

Rotors:
[[File:Picture5.png|frame|none]]

== Is there a 250W EPAC version of the DM01 ==
Yes. While you won't find it listed on their website yet, it is available on request. It is based on the 750W casing, which, along with the external controller exposed to airflow, provides superior thermal management to the smaller 250W DM02, plus some extra torque. It is meant as a "heavy duty" EPAC compliant motor for larger people and cargo bikes. Settings are locked and restricted to 250W continuous and 25km/h to comply with the EN15194 standard. Just keep in mind that EN15194 is a "whole bike" standard which takes into account frame strength and safety, braking capability and battery standards as well, so in some countries having a compliant motor may not be enough on its own.

Firmware is still being developed, as with all ToSeven motors, but so far it is pretty good. The speed limit is being tackled a little differently than other motors. Instead of a hard limit which can feel like a sudden "hitting a wall" loss of power, it tapers off more gradually. We should this new approach to a speed cut off transfer over to other mid drive motors as well. Maximum amps is set to 15 in Sport mode, which translates to over 800W of peak power with a fully charged 48V battery.
If pulled over for a roadside inspection, the bottom of the motor is etched with 250W EN15194, which should satisfy most cops, and it should pass a dyno test.
[[File:Picture204.jpg|frame|none]]

== Is it safe to use a 48-volt battery on the 250-watt DM02 ==
While [https://to7motor.com/product/dm02-ebike-mid-drive-motor ToSeven’s website] for all DM02 motors states “Compatible With 24, 36, 48, 52, 60 Volts Battery Range” *, the 250-watt version is not sold with a 48-volt option, only the 350-watt and 500-watt versions are. 

<nowiki>*</nowiki> Note that this is the voltage range that the controller is designed to accept. It does not mean that it is compatible with all battery voltages within that range without configuration. See [[#Should you configure the motor's voltage setting|Should you configure the motor's voltage setting]].

While it could just be a settings change (see “[[Display Settings#Battery|Battery settings]]”), doing so may make the bike technically illegal (Watts = volts x controller amps. 48 volts on the same controller will increase the wattage). These motors are also [https://discord.com/channels/713249006129512448/1093198162417688656/1134019225569198130 wound for 36V], so connecting a 48V battery is technically overvolting them. With that being said, overvolting a 36-volt motor with a 48-volt battery [https://www.google.com/search?q=48+volt+battery+on+36+volt+motor has been done plenty of times before] and it should be reasonably [https://discord.com/channels/713249006129512448/1093198162417688656/1252339125923745805 safe to do so].

== DM02 36V, 48V, 52V or 60V ==
'''Update''': ToSeven advises that ''"to ensure product reliability and enhance user experience, our factory has decided to temporarily pause the sales of DM motors (including DM01 and DM02) specifically configured for 60V batteries, effective from the beginning of this year [2025]. This decision was made after careful evaluation, as the 60V battery specification was found to impact controller longevity significantly."''

On To7’s AliExpress store and also from many resellers, you may find a choice of 36V or 48V for the 350 & 500W versions.
[[File:Picture13.png|frame|none]]
Order according to the battery you intend to fit, but there is no physical difference between these and this is just a way of advertising that the motors can do either. ToSeven confirmed this after someone enquired about fitting a 48V battery to a 36V version:

:''“Regarding Voltage and Parameters: The parameter 36V15A was configured based on the original order specifications when the motor was purchased.  Please note that our standard DM02 motor is compatible with both 36V and 48V batteries, and you can adjust the settings through the display to match your battery.”''

52V batteries are also supported, but it [[#Should_you_configure_the_motor's_voltage_setting|must be configured in the settings]] or the controller could be damaged. For battery selection and fine tuning in the settings, see “[[Display Settings#Battery|Battery settings]]”. However, while the sticker on the controller says “24-60V”, (see [[#What are the maximum safe amps for the 500W DM02|below]]) that means that the controller supports [https://discord.com/channels/713249006129512448/1093198162417688656/1339733366601289728 up to 60V but not a 60V battery since a fully charged 60V battery is close to 70V which exceeds the specs]. 

Note that the sticker or stamping on the bottom of the motors usually specifies watts, voltage and maximum amps. This may indicate different windings amongst other things.
{| class="wikitable"
| [[File:Picture14.png|frame|none]] 
| [[File:Picture15.png|frame|none]]
|}

== DM01 48V, 52V or 60V ==

'''Update''': ToSeven advises that ''"to ensure product reliability and enhance user experience, our factory has decided to temporarily pause the sales of DM motors (including DM01 and DM02) specifically configured for 60V batteries, effective from the beginning of this year [2025]. This decision was made after careful evaluation, as the 60V battery specification was found to impact controller longevity significantly."''

The DM01 does appear to have different controllers for different battery voltages as shown on [https://www.aliexpress.com/item/1005007826717311.html their AliExpress store]. 
[[File:Picture16.png|frame|none]]
With that being said, they have stated that a 60V controller will work fine with 48V or 52V batteries, you just adjust the voltage in the Battery settings, just not the other way around. i.e A 60V battery may not play nicely with a 48V or 52V controller, and may even damage it. On that basis, if you think you might want to use a higher voltage battery one day, buy the 60V controller version.

Nov 12, 2024 
:“''...the 60V controller is also compatible with 48V and 52V batteries. But you will need to adjust the motor battery voltage and the low cut-off voltage.''”

They have also said:

:“''If you wish to use a 60V battery, you will only need to replace the controller with a 60V version.''”

So we can safely assume that the controllers are different, and rated to a maximum voltage. 
Note also, on the bottom of recent motors are stickers specifying the voltage, maximum amps and wattage.
[[File:Picture17.png|frame|none]]

The controllers have 48-60V sticker, meaning they support voltages within that range.
[[File:Picture18.png|frame|none]]
These stickers appear on replacement controllers, while the controllers that ship with new motors are unmarked.
[[File:Picture19.png|frame|none]]
Below is a 72V controller clearly marked. The "48-72V" label on the controller sticker indicates the voltage range that the controller is designed to accept, not that it is compatible with all battery voltages within that range without configuration. A common misunderstanding is to interpret it directly as a battery voltage specification. Users must ensure the motor's configured voltage parameter matches their actual battery voltage.
[[File:Picture20.png|frame|none]]
72V controller beside a 48V controller. A larger capacitor is clearly evident.
[[File:Picture21.png|frame|none]]

== What are the maximum safe amps for the 500W DM02 ==
There is/was conflicting information about this. Originally, some resellers had a warning on their websites not to go above 15A after a couple of early controllers failed inexplicably. [https://discord.com/channels/713249006129512448/1093198162417688656/1286269982816210945 The manufacturer later advised that this issue had been resolved] and 18A was safe and fine. 
⁠The To7 website also said that the DM02 supported up to 18A with a 48V battery (up to 20A with a 36V battery):
[[File:Picture22.png|frame|none]]
The sticker on the controller inside the motor also says it supports 12-18A⁠ * :
[[File:Picture23.png|frame|none]]

<nowiki>*</nowiki> Note that "24-60V" is the voltage range that the controller is designed to accept. It does not mean that it is compatible with all battery voltages within that range without configuration. See [[#Should you configure the motor's voltage setting|Should you configure the motor's voltage setting]].

However, the manufacturer has since advised to ignore this and go by the sticker on the bottom of the motor, which in the case below is "DC48V'''15A'''":
[[File:Picture24.png|frame|none]]
This will ensure reliable and safe operation. Ensuring this is that firmware since [[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|November 2024 defaults to 14.2A]] with a 48V battery and is hard set. Even if changed to 18A, the motor will not go much above 14A while pedalling, even in the highest settings.

Note that the lower amps in later firmware versions make the motor feel somewhat breathless compared to previous firmware versions but the upside is it is less stressed and also appears to use a very clever formula to increase amps as battery voltage decreases. This means power doesn't drop off as the battery discharges, like most other motors do. The firmware also makes the torque sensor behave more reliably and solves “motor blocked” errors. For more information about why this amp cap was necessary and what it does, see ToSeven's explanation [https://endless-sphere.com/sphere/threads/technical-update-dm02-500w-firmware-optimization.129704/ here].

'''Note''': For recommended amps on other motors see “[[Example Setups#The Factory defaults (From Nov 2024 DM02/Dec 2024 DM01 firmware)|The Factory Default]]” settings

== Should you configure the motor's voltage setting ==
Yes! The label on the controller sticker (e.g "24-60V") indicates the voltage range that the controller is designed to accept. It does not mean that it is compatible with all battery voltages within that range, and not without configuration. A common misunderstanding is to interpret it directly as a battery voltage specification (nominal battery voltage). Generally speaking, when ToSeven refer to battery voltages in their specifications on both their webpage and the stickers on the components, they mean the upper and lower limits of voltage that the motor or controller or display will support, not the nominal battery voltage. For example, “24-60V” means it will support 36V, 48V and 52V batteries, but not 24V batteries since these are below 24V when fully discharged, and not 60V batteries since these exceed 60V when fully charged. If you wish to run a battery that falls outside of the advertised range, either fully charged or fully discharged, contact them for advice.

All T07 DM motors feature adjustable voltage settings. It is essential to ensure the motor's preset voltage parameter matches the nominal voltage of your battery pack. A mismatch can lead to performance issues or severe hardware damage. For how to do this, see [[Display_Settings#Battery|Display Settings > Battery]].

Why is this crucial? 

Using a battery voltage HIGHER than the motor's setting (e.g., using a 52V battery on a motor set to 36V) is extremely dangerous and can cause immediate, irreversible damage, including: 
* Instant Controller Burnout: The controller's internal components can be destroyed by the voltage surge. 
* Overheating & Fire Risk: Components will overheat, potentially melting cables or causing a short circuit. 
* Motor Demagnetization: The motor's magnets can be permanently damaged, leading to a permanent loss of power and efficiency. 

Using a battery voltage LOWER than the motor's setting (e.g., using a 36V battery on a motor set to 52V) will not damage the motor instantly but causes significant issues:
* Poor Performance: The motor will feel weak, with slow acceleration and an inability to reach top speed. 
* Overheating Damage: The controller and motor will overheat from drawing excessive current, leading to a slow failure. 
* Battery Damage: The battery will be stressed by the high current draw, shortening its lifespan. 
Before your first ride, always verify that the motor's voltage setting is correct for your battery.

== How to determine the production date of the motor ==
On the bottom of the motor will be a string of numbers or a QR code. In the case of the QR code pictured above, it is '''TO 02 S 050 2409 00017''' (spaces added for clarity), which indicates the following:

'''TO''' = ToSeven

'''02''' = means it’s a DM02 series (II for DMII), 01 for DM01

'''S''' = bottom bracket width of 68-73mm. L is 100-110

'''050''' = 500W. 035 = 350W, 075 = 750W etc.

'''2409''' = September 2024 production date

'''00017''' = product serial No.

== Is there a difference between the DM02 and DMII ==
To7 refers to recent motors since October 2024 as “DMII”, and older motors as “DM02”. This indicates a possible revision change with some minor internal upgrades, but they still use the same firmware, displays and sensors. Note that the DMII motors come without access to the Advanced menu settings. (To7 have indicated that they will provide instructions to unlock them on request, with the understanding that doing so will void warranty).

== Why is there more than 500 watts showing on the display while riding the 500W DM02 ==
This is true for all eBikes and kits. Electric motors are rated at nominal power rather than peak power. Nominal power refers to the continuous, sustainable power output a motor can produce without overheating, while peak power represents the maximum burst of power the motor can deliver for a short period. To fit legal requirements, and also make their motors feel more powerful than a competitor’s similarly specced motor, manufacturers use the nominal power rating for motors. However, in reality, the motors produce much higher wattages, at least for short bursts before they start getting hot.
A basic formula for peak power is: volts x controller amps = watts. So the 500W DM02 with amps set at 18 with a 48V battery is: 48  x 18 = 864 watts. A fully charged 48V battery has 53 volts, making up to ~950 watts possible. 
Here is [https://www.emtbforums.com/threads/pw-x2-power-torque-curve.17918/#post-243869 a chart from 2018 comparing watts vs revs] of various popular “250W” motors.
[[File:Picture25.png|frame|none]]

== Is the 500-watt DM02 enough power ==
That is a subjective question. 500 watts is a decent amount of power to make getting up hills and maintaining speeds along flats a lot less effort, while still feeling like a bicycle rather than a moped. Most people should find it more than adequate. If you want something with a lot of power, go for the [https://to7motor.com/product/dm01-e-bike-mid-drive-motor DM01] instead.

== Is there a rider weight limit for the DM02 ==
There is no official rider weight limit thus far. However, many bike manufacturers specify 120kg (260lbs) for their bikes, which would be a reasonable figure to go by for a lot of ebike motors including the DM02. This could mean running at higher temperatures or perhaps “motor blocked” errors from time to time. (More on this [[Troubleshooting#What does the “Motor Blocked” error message mean |here]]). If you’re on the heavier side of the scale, or carrying heavy loads such as a cargo bike, it is recommended to go for the [https://to7motor.com/product/dm01-e-bike-mid-drive-motor DM01] instead as it is more likely to cope.

== Is the DM02 tough enough ==
[https://www.youtube.com/@HighVoltageKits Captain Codswallop] has been torturing the DM02 by installing an aftermarket controller which allowed him to increase the wattage by up to 2500, which is five times above what it was designed to handle. After 500 kms, the only failure was the blue nylon drive gear, which is doing what it’s supposed to do. i.e. acting as a failsafe rather than destroying more expensive parts, as well as being a lot quieter than metal on metal gears. While most people would start with a more powerful motor to begin with for this kind of application, this gives us some insight into what the limits of this little motor are. Check out the following videos:
* [https://youtu.be/Zgt0r7YKPJ8?si=Lr2ZD-XoU-1IXzwT How much power can this tiny mid-drive diy motor handle? TOSEVEN motor DM02]
* T[https://youtu.be/-CveMyyaEWE?si=KLqMOcKXtjw0g1Og iny Mid Drive Motor - Testing the TOSEVEN DM02 at 2.5kw]
* [https://youtu.be/aJethPk-QI8?si=2yhBM-gcs9FyEBth High Power testing of the DM02 with the ASI BAC855]
* [https://youtu.be/K64o6hHBfbg?si=SR9eYdyt4AHGCajt Finally broke the DM02 with the ASI BAC855]

== Are they waterproof ==
ToSeven motors and displays are IP65 rated, which means they will keep dust and water spray out but they are not rated for complete submersion. This means they are safe enough to wash with a hose or ride in the rain, but it is advisable to avoid heavy rain, using a water blaster to wash it, or to ride through anything deeper than a couple of inches. There are at least two reports of controllers failing after being exposed to these conditions. However, it is suspected that lack of waterproofing of the XT60/XT90 connector may have been the cause, or possibly a sudden hit of cold water on a hot motor.

It is recommended to heat shrink the individual contacts going into XT60, XT90 and Anderson connectors to prevent shorting, particularly from water ingress. This is generally not done by default from the factory. Alternatively, use silicone if the connector can not be redone easily. It is not recommended to heat shrink both wires together as this can trap moisture and increase the possibility of shorting.
[[File:Picture26.png|frame|none]]
After disassembling and reassembling the motor to make repairs, it is also advisable to add silicone sealant around the entry point for the wires, and ensure that the rubber gaskets are in good condition and correctly installed.

== Manuals and other documents ==
Some of the manuals and other documents can be tricky to find. The following links are copies to make it easier, or in case they disappear. Note that some of them may be a little out of date. Some others are available on [https://discord.com/channels/713249006129512448/1100260764704571482 Discord]. There is also an LCD-135 manual [https://endless-sphere.com/sphere/threads/toseven-lcd135c-dm01-display-manual.124329/ here].
* [https://drive.google.com/file/d/13UmK75G8CCo-wRcFzgS1HcRMR0jrk1DR/view?usp=drive_link DM02 User Manual]
* [https://drive.google.com/file/d/13VJz1Fyfxf4y5YP4ls9HtXtQHLl-_MIJ/view?usp=drive_link DM01 User Manual]
* [https://drive.google.com/file/d/140q849oiCjUna2qlQ6E8KQNkbvefX4wS/view?usp=drive_link T24 Display Manual User Guide]
* [https://drive.google.com/file/d/13aa6WF2ImfEg85jUSPUoJVUndqp2LWhh/view?usp=drive_link T24 Display Manual (newer)]
* [https://drive.google.com/file/d/13uCCAp66bY0Oh_REHZhindwXMD9flcCy/view?usp=drive_link T24 Display Setting Interface]
* [https://drive.google.com/file/d/13zO6haZKSTfyBGb0s7ov2IjS-oAfJlNI/view?usp=drive_link T24 Operating Instructions]
* [https://drive.google.com/file/d/13sgYn5eWR03jnVoOjfR4RgWECmkY6wV7/view?usp=drive_link T154 Display Manual User Guide]
* [https://drive.google.com/file/d/1xVcvkm1mD7BQ0pr5z58vqLzsDDxPIgcE/view?usp=sharing LCD135C User Manual]
