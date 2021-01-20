:objects child1 child2 - child
:objects bread1 bread2 - bread-portion
:objects content1 content2 - content-portion
:objects tray1 tray2 - tray
:objects table1 table2 kitchen - place
:objects sandwich1 sandwich2 - sandwich

:init at tray1 kitchen
:init at tray2 kitchen
:init at_kitchen_bread bread1
:init at_kitchen_content content1
:init at_kitchen_bread bread2
:init at_kitchen_content content2
:init no_gluten_bread bread1
:init no_gluten_content content1
:init allergic_gluten child1
:init not_allergic_gluten child2
:init waiting child1 table1
:init waiting child2 table2
:init notexist sandwich1
:init notexist sandwich2

:goal served child1
:goal served child2