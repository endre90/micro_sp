:objects rooma roomb - room
:objects ball4 ball3 ball2 ball1 - ball
:objects left right - gripper

:init enum at-robby rooma
; :init bool free left
; :init bool free right
:init enum at ball4 rooma
:init enum at ball3 rooma
:init enum at ball2 rooma
:init enum at ball1 rooma

:goal enum at ball4 roomb
:goal enum at ball3 roomb
:goal enum at ball2 roomb
:goal enum at ball1 roomb