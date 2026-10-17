# Batteries

Source: https://toseven.miraheze.org/wiki/Batteries

== What to look for in a battery ==
Any battery that can handle at least a 20-amp draw will be fine for the DM02 (30+ amps for the DM01)*, but more is quite okay. For example, a 48V 40-amp battery is fine to use on the 500W DM02 which draws a maximum of 18 amps. Just not the other way around. A 10-amp battery will not work well, if at all. (Note, we’re not talking amp hours (ah) here. That is the battery capacity, not the number of amps that can be drawn at once).

<nowiki>*</nowiki> ToSeven warns that batteries with a lower draw than the controller can demand may damage the controller and/or may trigger the battery's BMS and make it shut down:

:''"Recently, one of the users reported an issue while using our DM motor on flat city streets under dry early autumn conditions with temperatures around 20°C. During the ride, the display suddenly reset and showed the error message: "Controller Broken."''

:''This user was using a 14S4P battery with a continuous current rating of 20A and a maximum burst current of 27A, paired with our 1000W motor. We would like to clarify the cause of the controller damage based on this information:''

:''Our 1000W controller is rated for a maximum current of 30A, while the battery’s maximum burst current is 27A. When the battery output current exceeds 27A, its Battery BMS activates to cut off power output to protect the battery from overload or excessive discharge.''

:''If the motor frequently disconnects and reconnects to the battery during operation due to this BMS action, this might cause reverse voltage spikes in the MOSFETs inside the motor controller. Over time, this can potentially damage the controller.''

:''To avoid frequent triggering of BMS protection and reduce the risk of controller damage, we strongly recommend all users ensure that their batteries have a maximum current rating close to or higher than the controller’s maximum current."''

As a rule of thumb:
* For 500W or lower motors with up to 20 amp controllers, a battery with at least a 20 amp BMS current is needed
* For 750W motors with up to 25 amp controllers, a battery with at least a 25 amp BMS current is needed
* For 1000W motors with up to 30 amp controllers, a battery with at least a 30 amp BMS current is needed
* For controllers above 30 amps, well you get the picture.

Voltage-wise, higher voltage batteries have better range and provide more power than lower voltage batteries, but they are more expensive. Likewise, a 20ah battery has better range than a 13ah battery but it will also be more expensive. It may also be physically larger and may not fit inside your bike frame. Be aware also that [[General FAQs#DM02 36V, 48V, 52V or 60V|different voltage batteries may require ordering a different controller version]].

A simple formula for [[General FAQs#Why is there more than 500 watts showing on the display while riding the 500W DM02|peak power]] is volts x controller amps = watts. So a 48v battery with a 30A controller will give up to 1590W with a full (53V) charge. 52V, up to 1720W, 60V up to 1970W, and so on. However, you won't be using maximum power all of the time, and that peak power will drop as the battery discharges.

A simple formula for range is volts x battery amp hours (ah) = watt hours.
Watt hours divided by 12.5 (12.5w per km) = maximum range (ridden conservatively  in the lowest power level on smooth, flat roads).

Battery watt hours divided by motor watts = time at full watts.
So, a 13ah 48v battery = 624wh. 624 ÷ 12.5 = up to 49 km if ridden carefully (divide by 1.6 for miles). Ridden hard, 624wh with a 1000w motor (624 ÷ 1000) = .624 hrs (.624 x 60 = 37 mins).

There is a spreadsheet calculator for the above [https://docs.google.com/spreadsheets/d/13qfLxWew7bl1q6QYHnsxjv61Wk4qd1Ku/edit?usp=drive_link&ouid=101064065116260547277&rtpof=true&sd=true here]. (Download or copy to your own Drive folder, or ctrl-z repeatedly to return it to its original values when finished with it). Only alter the values in red as applicable, and the rest will automatically update. Keep in mind that these figures are just a ballpark to get a rough idea of how a battery might perform with a given motor and controller. Weight, terrain, tyre width & tread, power level, rider’s effort and so on, will all impact range.

Another easy to use online battery range calculator can be found [https://reallygoodebikes.com/pages/electric-bike-ebike-range-calculator here].

Where you buy your battery is also important. The old saying “if it seems too good to be true, it probably is” applies. Suspiciously cheap batteries use low quality no-name cells and may not have an adequate protection circuit/BMS (Battery Management System). These are generally the ones that [https://youtu.be/TKeW6Mcxr_c?si=RiE6uipEhZxomjvW catch fire].

== Will you also need a battery carrier ==
[[File:Picture123.png|frame|none]]

Yes, you will. ToSeven kits and most other kits on the market do not come with battery mount/carriers since there are lots of different batteries out there which all use different carriers. 

Battery carriers also come with different connectors to connect them to the motor. 

ToSeven will fit whichever connector type you need when you order from their AliExpress store, but you will need to manually request it on the order form when you place the order. I didn’t do this, and it came with an XT60 connector, but fortunately it matched my carrier anyway.
[[File:Picture124.png|frame|none]]

If you don’t do this and find yourself with a mismatch, you can usually buy an adapter. Or you can buy a pigtail, cut the incorrect connector off and twist & solder the pigtail in its place.

'''Note 1:''' It is recommended to heat shrink the individual contacts going into XT60, XT90 and Anderson connectors to prevent shorting, particularly from water ingress. This is generally not done by default from the factory. Alternatively, use silicone if the connector can not be redone easily. It is not recommended to heat shrink both wires together as this can trap moisture and increase the possibility of shorting.
[[File:Picture26.png|frame|none]]

'''Note 2:''' Battery carriers mount to the bottle cage bolts on the frame. These are not particularly strong. For offroad use it is generally recommended to add a third bolt. This may also be needed if the cage bolts are not in a suitable position on the frame. This will require a right-angle drill chuck, a rivnut (aka nutsert) and a rivnut tool. See [https://youtu.be/N-eMTKRW7aw?si=GsY7N7elTvf6NN9A here] for an installation example. These can all be found relatively cheaply on eBay, Amazon and AliExpress, or ask a bike shop to do it.

Another option is the [https://ebikes.ca/product-info/grin-products/battery-anchors.html Grin Tech Battery Mounts] which provide extra anchor points without any drilling.

== Does an XT90 connector fit through the opening on the DM02 ==
Yes it does. Pictured is an XT90 in the hole and an XT60 in hand.
[[File:Picture125.png|frame|none]]

== Is it necessary to configure the battery voltage settings ==
Yes! Having a mismatch between the configured battery voltage and the actual battery voltage can cause damage and can even be dangerous. See "[[General FAQs#Should you configure the motor's voltage setting|Should you configure the motor's voltage setting]]".

== Are any precautions necessary when connecting and disconnecting the battery ==
If your battery has an on/off switch, it is recommended to switch it off when connecting or disconnecting it from the carrier, or unplugging it. If the battery is left on, it can create a condition called Current In-Rush and/or an arc (spark) may occur, which can cause damage to the controller over time. The following video from Johnny Nerdout explains more:

<embedvideo service="youtube" width="640" alignment="center">zc18LjinrOQ</embedvideo>

When charging the battery, it is also a good practice to connect the charger prior to turning the power on to it.

== Battery doesn’t show as 100% full when fully charged ==
Battery calibration can be fine-tuned in the [[Display Settings#Battery|Advanced Settings]] section * under “Battery”. While in this menu, long press the up and down arrows together for more options. 
[[File:Picture126.png|frame|none]]

Select each % to adjust by pressing the up/down arrows to scroll, then short press the power button to select, then up/down again to adjust. See Advanced Settings > battery below for recommended values.

<nowiki>*</nowiki> In more recent firmware versions, the battery settings are found under [[Display Settings#Device|Main Settings > Device]].
