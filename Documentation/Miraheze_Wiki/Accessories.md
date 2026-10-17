# Accessories

Source: https://toseven.miraheze.org/wiki/Accessories

== Are Bafang brake sensors, throttles and speed sensors compatible ==
No. Well, not out of the box. They use different pinouts so you will need to cut and rejoin the wires to make the pinouts match. More on this in [https://youtu.be/aOgo0OkjcyI this video] from around 12:08 or the “[https://discord.com/channels/713249006129512448/1093198123821694976/1380643595438129194 napkinCAD]” below.
[[File:Picture7.png|frame|center]]

One example is [https://www.aliexpress.com/item/1005010082926124.html this thumb throttle] which was successfully adapted to the existing ToSeven Julet connector with the following splicing:

From the bike - 
* White to red
* Red to green 
* Black to black
[[File:Picture201.jpg|400px|thumb|center]]

Here is the colour code for a Sur-ron throttle successfully adapted to a ToSeven throttle cable.
[[File:Picture203.jpg|400px|thumb|center]]

Another method that can work with [https://www.magura.com/en/EUR/bicycle_components_s-pedelec_brakes_mt5e_higo/p/mt5e Magura MT5e Hydraulic brake levers] for ebikes is to [https://discord.com/channels/713249006129512448/1093198123821694976/1337569507123658762 cut out the keyway and rotate the plug]. Be careful not to bend the pins. Once in place and working, it may be worthwhile marking the position to make it easier to reconnect again if you ever need to pull it apart.
[[File:Picture8.png|frame|center]]

== Standard Higo/Julet pinout chart ==
[[File:Picture95.png|frame|center]]

== ToSeven Higo/Julet pinout chart ==
[[File:Picture96.png|frame|center]]
[[File:Picture97.png|frame|center]]

== Are Bafang displays compatible ==
No. As above, they almost certainly use different pinouts, and may not work well anyway. At best, you will lose the ability to change settings. ToSeven has a large number of displays to choose from [https://to7motor.com/ebike-display here]. Some are listed on [https://www.aliexpress.com/store/1103813115/pages/all-items.html their AliExpress store], or are available as options with the motor kits. Others are probably available on request. Note that some displays may be limited to a maximum of 48 volts.

== Is it possible to install a shift sensor on the DM02 ==
A shift sensor has the gear shifter cable passing through it and detects when it moves. When it does, it cuts the motor power for a moment, allowing a safer gear shift. More info in [https://www.electrifybike.com/blogs/news/do-you-need-a-gear-shift-sensor this blog].
[[File:Picture98.png|frame|none]]

The DM02 controller does not come with a connector for a shift sensor so it is not possible to install one. (The DM01 motor does come with one, which likely has different pinouts to Bafang shift sensors so they won’t work without cutting and rejoining wires). However, a shift sensor is not needed with the level of power that the DM02 produces, and the torque sensor allows reducing the power to allow a safe shift anyway. Simply reduce pedalling effort and it will shift just fine. Alternatively, back pedalling slightly just before you shift will cut the power temporarily, making for perfect shifts. Likewise with internally geared hubs such as Alfines and Rohloffs which do not even require the chain to be moving to shift. With a little practice with the pack pedalling method, lightning-fast shifts are possible, even under full power.

Another method is to lightly squeeze one of the brake levers enough to trigger the brake sensor, then shift, then release (a little like using a motorcycle clutch). Alternatively, one of the brake sensors can be modified to work as a manual kill-switch (anything that moves the magnet away from the sensor and back). Bafang kill switches like these below likely have different pinouts and will need to be respliced to work.
[[File:Picture99.png|thumb|none]]

However, as mentioned, they are not needed. Just remember to reduce pedalling effort or back pedal slightly just before shifting.

== ToSeven Display comparison ==
Below is a quick comparison of the [https://to7motor.com/ebike-display displays listed on ToSeven’s website]. Highlighted are options for the DM02 on their AliExpress store. Others in the table should be ok as long as they support the UART protocol and have UART connectors, but check with ToSeven before buying. 
[[File:Picture100.png|frame|center]]

Note that while the DM02 can support battery voltages from 24-60V, some displays only go up to 48V. [https://discord.com/channels/713249006129512448/1277192441233014876/1278810080187383911 All current T24 displays should support up to 60V] but some older ones may not. There are also several assumed typos on their website so there may be an error or two if I have assumed incorrectly. One user also reports that it says “Max: 60V” [https://discord.com/channels/713249006129512448/1093198162417688656/1329581315959427083 on the back of their T154v3], so the webpage specs may not always be accurate or up to date. However, if asked, [https://discord.com/channels/713249006129512448/1093198162417688656/1363644345537790052 they still recommend max: 48V]. Furthermore, “Max: 60V” probably doesn't mean it will support a 60V battery. Rather, it is likely the maximum voltage it will support. A fully charged 52V battery is 58.8V so that is probably the largest battery it will handle. A fully charged 60V battery is 67.2V, which exceeds the limit.

{| class="wikitable" style="width:800px; text-align:center; white-space:nowrap;"
|-
! Display
! Screen Size
! Type
! Supported protocols
! Bluetooth
! Supported voltages
|-
| style="background-color: #ffff99;" | T24
| style="background-color: #ffff99;" | 2.4"
| style="background-color: #ffff99;" | Colour TFT
| style="background-color: #ffff99;" | UART/RS232/CAN
| style="background-color: #ffff99;" | N
| style="background-color: #ffff99;" | 15-60V
|-
| LCD-135 B/W
| 3.5"
| Monochrome
| UART/RS232/CAN
| N
| 15-90V
|-
| LCD-135CBT
| 3.5"
| Colour TFT
| UART/RS232/CAN
| Y
| 15-90V
|-
| LCD-135C
| 3.5"
| Colour TFT
| UART/RS232/CAN
| N
| 15-90V
|-
| style="background-color: #ffff99;" | T154
| style="background-color: #ffff99;" | 1.54"
| style="background-color: #ffff99;" | Colour TFT
| style="background-color: #ffff99;" | UART/RS232/CAN
| style="background-color: #ffff99;" | N
| style="background-color: #ffff99;" | 24-48V
|-
| style="background-color: #ffff99;" | M15
| style="background-color: #ffff99;" | 1.54"
| style="background-color: #ffff99;" | Monochrome
| style="background-color: #ffff99;" | UART/RS232/CAN
| style="background-color: #ffff99;" | N
| style="background-color: #ffff99;" | 24-48V
|-
| style="background-color: #ffff99;" | C1
| style="background-color: #ffff99;" | 3.5"
| style="background-color: #ffff99;" | Colour TFT
| style="background-color: #ffff99;" | UART/RS485/CAN
| style="background-color: #ffff99;" | N
| style="background-color: #ffff99;" | 24-72V
|-
| C2
| 3.5"
| Colour TFT
| UART/RS485/CAN
| N
| 24-72V
|-
| C3
| 1.77"
| Colour TFT
| UART
| N
| 24-72V
|-
| E100
| 
| LED
| UART/RS232/CAN
| N
| 15-60V
|-
| L1
| 1.8"
| LED
| UART
| N
| 24-48V
|-
| S2
| 3.5"
| Colour LCD
| UART/RS485/CAN
| N
| 24-72V
|-
| S3
| 2.5"
| Colour LCD
| UART/RS485/CAN
| N
| 24-72V
|-
| S4
| 2.2"
| Colour LCD
| UART/RS485/CAN
| N
| 24-60V
|}

== LCD-135C Display and the Android app ==
The LCD-135C comes in three versions - a monochrome and two colour versions, one with Bluetooth and one without. Be sure to order the correct one if you want Bluetooth. The Bluetooth version can be identified by the letters “BLE” on the back.
[[File:Picture101.png|frame|none]]

Note that the throttle is connected to the "TH" socket on the back, which has a different connector than needed for the T24 display which connects directly to the controller using a Julet/Higo connector (the yellow one in the image below).
[[File:Picture190.png|thumb|none]]

The Android app for it can be downloaded from To7’s website [https://to7motor.com/progam-updating-instructions#display here] just under the LCD-135C firmware button. However, [https://discord.com/channels/713249006129512448/1322979026570051727 there are reports of it not working with Android versions below Android 13]. This is reported to be fixed now so try it first, but if it still happens, try an older version:

* [https://drive.google.com/file/d/14-wnsvCry3znyqDapPp0q38RIPirCmoq/view?usp=drive_link April 2024] (confirmed to work in Android 11)
* [https://www.mediafire.com/file/gcl43xdhcvh1qys/EBike_TOSEVEN_1-1.07-SDK33(1).apk.1.1.1/file July 2024] (This one requires renaming the file to end in .apk) 
* [https://to7motor.com/wp-content/uploads/2024/10/TOSEVEN1.zip Oct 2024] 
* [https://to7motor.com/wp-content/uploads/2024/11/EbikeTOSEVEN.zip Nov 2024]

Note that the file needs to be unzipped first, and the apk file is not supported by Google so it will need to be sideloaded. Instructions [https://www.xda-developers.com/how-to-sideload-install-android-app-apk/ here] if you have not done that before. This is for Android only. There is not an iOS version available yet.

== Are Bafang Chainrings such as Lekkie Bling Rings compatible ==
No, these have a different bolt pattern. At this stage you can only fit the ToSeven-supplied spider. However, these will take a standard 110BCD 5-bolt chainring. Lekkie do plan to release chainrings for ToSeven motors in due course. Cheaper but less attractive and robust copies will almost certainly appear on AliExpress as well if there is sufficient demand for them.

== Do Tongsheng spiders fit ToSeven motors ==
No, ToSeven motors (left) have a centre spline and locknut. Tongsheng spiders are attached with five centre bolts (right). The chainrings are both 110BCD though.

{| class="wikitable"
| [[File:Picture102.png|frameless|none]] 
| [[File:Picture103png.png|frameless|none]]
|}

== Does the Lekkie One Nut fit To7 motors ==
The [https://lekkie.tech/product-category/fitting-hardware/ Lekkie One Nut] is designed for Bafang motors which have a smaller thread size. The DM02 and DM01 have a 33.5mm thread, specifically M33.15x1.5, which is the same as the TSDZ2. Lekkie has One Nuts, which they have called [https://lekkie.tech/product/onenut-tongsheng-lock-nut-gunmetal-grey/ 1TSNuts, to suit ToSeven and Tongsheng motors now available]. Here is an 1TSNuts fitted to a DM02:
[[File:Picture104.jpg|600px|thumb|none]]
And here it is on a DM01:
[[File:Picture188.jpg|600px|thumb|none]] 

The advantage in these is that they use a standard Shimano bottom bracket tool or socket so they are easier to torque down, and they have two grub screws that prevent it from coming loose.

'''Note that turnbuckles as torque arms are not recommended [https://toseven.miraheze.org/wiki/Installation_and_bike_fit#Home_made_torque_arms if the motor is not hard up against the downtube as they are not designed to resist pushing forces, only pulling].'''

== What lights are compatible ==
=== Lights controlled from the display. ===
More recent versions of the DM02 come with a connector and Y cable to connect lights. Older versions will need to replace the controller to get the connector. The supplied cable has a green 5-pin Julet (aka HiGo) connector for the controller end, and red 2-pin Julet connectors for the lights. Unintuitively, the shorter (900mm) cable is for the head light, while the longer one (1200mm) is for a tail light.* These can then be [[Display Settings#Light|turned on/off via the display or set to come on automatically]]. The tail light can also function as a brake light as described under [[Display Settings#Device|Main Settings > Device > Brake Light]].

<nowiki>*</nowiki> '''Note''': Be sure to connect the cables the right way around. See "[[Troubleshooting#Headlight flashes or strobes when brakes engaged|Headlight flashes or strobes when brakes engaged]]".

ToSeven have their own compatible headlight [https://www.aliexpress.com/item/1005007796441584.html here], and a tail light [https://www.aliexpress.com/item/1005007793361054.html here]. 
[[File:Picture105.jpg|frame|none]]

Be aware also that these are only 6-volt which indicates there’s only 6 volts to play with. They appear to be the same as Bafang lights which you can see reviewed [https://youtu.be/If65KMUOuk8?si=ny_9k49ctbPEA2KN here]. Going by that, they should be fine for road use but probably not the brightest for off road riding. For off road, stick to recommendations in MTB forums. 

Ian’s review:

:''I like the To7 light set up for standard urban commuting and riding under 25kph, the light is surprisingly bright and has a brake light functionality if you have the brake sensors set up. It is not good, nor is it designed for, dark trails, country roads and speeds over 30kph etc The lens has very clear cut-off lines so there is limited peripheral vision and does not 'throw' light far enough for riding at speeds.''

Finding other lights with these connectors can be difficult but you may find other candidates by searching for [https://www.aliexpress.com/w/wholesale-ebike-light.html “ebike light” on AliExpress] and scanning for the right connector. Also ensure it supports 6-volts or you will require a relay to make it work. A 12-volt LED light on a 6-volt power supply, for example, probably won’t work. If it does, it will be very dim. Other candidates may show up in the “related items” that come up under each one. Also ignore advertised lumen values. These are almost always grossly exaggerated. Instead, read the reviews. 

If you find a suitable light but without Julet connectors, it is not too difficult to find J[https://www.aliexpress.com/w/wholesale-julet-connector.html?spm=a2g0o.detail.search.0 ulet connector pigtails] that you can twist and solder (and heatshrink) onto any DC light. (Tip: Solder Butt Connectors make this task dead easy. See [https://youtu.be/lmfiOJjRwRk?si=bC-kzxjLysUQUhnE here]. However, they do have their critics who prefer [https://youtu.be/NSqPHQ1zQco?si=NhtW8fjL5jewOrZz the old-school method].)
 
Pigtails:
[[File:Picture106.png|frame|none]]

For higher powered lights, you can add a splitter in between the motor and battery. To use the display to control the lights, you'll need a pigtail to connect a relay to the 6-volt connector, and then the lights get power through the relay, from the splitter. If you're doing direct control of the lights, the splitter would go to your light control interface (buttons on the bar, I'm assuming), and the lights get power from that. For more detail, see [https://forums.electricbikereview.com/threads/hunter-light-upgrade.46059/ here] and [https://forums.electricbikereview.com/threads/jugernaut-ultra-1000-dutch-commuter-style-w-integrated-lights.29178/ here].

=== Lights powered by the battery ===
Below is a diagram for how to wire a 12V light to a 48V battery.

[[File:Picture205.jpg|frame|none]]

Parts List (If any of these disappear just search for the same name on AliExpress):
* [https://www.aliexpress.com/item/1005007435588118.html 48V to 12V DC to DC Converter (10A)] 
* [https://www.aliexpress.com/item/1005006856749915.html XT60 1-Male to 2-Female Y Splitter] 
* [https://www.aliexpress.com/item/1005009156026988.html XT60 Male pigtail] 
* [https://www.aliexpress.com/item/1005003765026675.html 16 AWG cable] 
* [https://www.aliexpress.com/item/4001243079952.html 10A fuses] 
* [https://www.aliexpress.com/item/1005007030764216.html 3-Way Switch] 

Notes:
* There are many variations. For example, the 10A fuse can be anywhere on the positive wire in between the pigtail and the DC to DC convertor, depending on where it is more convenient to put it.
* While it may be possible to wire these in to allow the display to turn them on and off, it is generally easier to wire them to a separate switch. 
* Rather than splitting the e-Bike battery, a 12V power pack can replace the DC to DC convertor. I connected the output from the DC to DC convertor to a [https://www.aliexpress.com/item/1005006255392640.html 12V 5.5x2.1mm plug] to allow me to do this easily if I wanted to convert the bike back to analogue but keep the headlight functional.
* The wires from the 3-way switch and the inputs to the headlight could be all kinds of colours and may involve some trial and error to figure out which is high beam and which is low beam.
* All connectors should be [https://youtu.be/uj%20PbRBirkQ twisted and soldered] and [https://www.aliexpress.com/item/1005007077110130.html heat shrunk]. (You will need a soldering iron, electrical solder, some soldering rosin is recommended, and a heat gun makes the job easy for heat shrinks. 
* The e-bike battery should be switched off when not in use in case of parasitic drain.
* The low beam on some of the AliExpress 12V LED Motorcycle headlights is not very bright but is still useful as a daytime running light or when riding in well lit areas when you don’t want to blind cars. The high beam isn’t stupidly bright, but bright enough to dazzle drivers.
* To keep it simple, the daytime running light wires can be ignored if the light comes with them.
[[File:Picture110.png|frame|none]]

=== Flashlight mounted lights === 
If you have a lot of bikes, it is more cost effective to have lights that can be easily moved from bike to bike. It is hard to go past a powerful flashlight mounted to the handlebars in this case. While there are many to choose from, my preference is the Convoy S2 with Biscotti firmware since it is very bright, has a great beam pattern for cycling, it’s small and light, and it has many configuration options. While this can't be turned on or off using the display, it is easily reached on the handlebars or on your helmet.
[[File:Picture111.png|frame|center]]

What you will need:
* [https://aliexpress.com/item/32969287656.html Convoy S2]. I go for the 6500K (white) colour, and 7135x4 driver if I can get it. These will run at full power without getting overly hot, and are plenty bright enough. If I can't get them, the 7135x8 version is even brighter but it will get very hot after a while at full power so I usually run them at 35-50% on the bike, which is still seriously bright. Buy a second one for your helmet for offroad use. (The configuration options are in the description at the above link).
* Two or more rechargeable 18650 batteries (buy locally and go for quality brands like Panasonic)
* An 18650 battery charger
* GoPro handlebar mounts and/or a helmet mount (cheap and easy to find on AliExpress for all bar sizes. Buy one for each bike.)
{| class="wikitable"
| [[File:Picture112.png|thumb|none]]
| [[File:Picture113.png|thumb|none]]
| [[File:Picture114.png|thumb|none]]
|}

* GoPro flashlight mounts - Buy one for each bike. Dirt-cheap on AliExpress (I drill a hole through them and use a reusable zip tie to secure them rather than the supplied silicone band so it doesn’t wobble or risk breaking and falling off)
{| class="wikitable"
| [[File:Picture115.png|thumb|none]]
| [[File:Picture116.png|thumb|none]]
|}
* For a tail light that can easily be moved between bikes, any cheap rechargeable one, or that can take rechargeable batteries, will do. There are many that come with silicone bands to attach to a seatpost, or alternatively can be clipped onto the back of a saddle bag, backpack, or a helmet by drilling a hole through the vents and adding a zip-tie. 
{| class="wikitable"
| [[File:Picture117.png|thumb|none]]
| [[File:Picture118.png|thumb|none]]
| [[File:Picture119.png|thumb|none]]
|}

== Is there a bash guard available ==
Thostr designed a 3D printable cover for the DM02 [https://www.thingiverse.com/thing:6633663 here]. 
[[File:Picture120.jpg|frame|none]]

It comes with the warning: ''Be aware that this cover will reduce air flow over the motor and increase internal temperature. Use with caution. I made this for use in cold places (where I live...) and on wet and muddy days.''

There is also one for the DM01 available [https://www.thingiverse.com/thing:6826435 here].
[[File:Picture121.png|frame|none]]

ebikestuff.eu also has [https://ebikestuff.eu/en/products/321-cable-guard-protector-for-toseven-dm01-motors.html a cable guard for the DM01].
[[File:Picture122.png|frame|none]]

[https://discord.com/channels/713249006129512448/1093198123821694976/1475626996833976371 This custom design for the DM01] incorporates a bash guard and cable guard all in one.
[[File:Picture196.jpg|400px|thumb|none]]
