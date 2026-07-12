while true do
    screen:clear(Color.new(24, 18, 62))
    screen:fillRect(105, 70, 270, 132, Color.new(105, 70, 215))
    screen:fillRect(125, 90, 230, 92, Color.new(18, 22, 36))
    screen:fillRect(145, 115, 190, 42, Color.new(145, 105, 255))
    screen.flip()
    screen.waitVblankStart()
end
