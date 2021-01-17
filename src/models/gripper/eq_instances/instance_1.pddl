:objects rooma roomb - room
:objects ball1 ball2 - ball
:objects left right - gripper

:init enum at-robby rooma
:init bool free left
:init bool free right
:init enum at ball1 rooma

:goal enum at ball1 roomb