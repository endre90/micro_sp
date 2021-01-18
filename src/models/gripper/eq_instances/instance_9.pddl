:objects rooma roomb - room
:objects ball9 ball8 ball7 ball6 ball5 ball4 ball3 ball2 ball1 - ball
:objects left right - gripper

:init enum at-robby rooma
:init bool free left
:init bool free right
:init enum at ball9 rooma
:init enum at ball8 rooma
:init enum at ball7 rooma
:init enum at ball6 rooma
:init enum at ball5 rooma
:init enum at ball4 rooma
:init enum at ball3 rooma
:init enum at ball2 rooma
:init enum at ball1 rooma

:goal enum at ball9 roomb
:goal enum at ball8 roomb
:goal enum at ball7 roomb
:goal enum at ball6 roomb
:goal enum at ball5 roomb
:goal enum at ball4 roomb
:goal enum at ball3 roomb
:goal enum at ball2 roomb
:goal enum at ball1 roomb