package geneuron.creature

class Physics(_x:Float, _y:Float, _vx:Float, _vy:Float, _deg:Float, _energy:Float) {
  var (x, y, vx, vy, deg, energy) = (_x, _y, _vx, _vy, _deg, _energy)
  def update():Unit = {
    x += vx
    y += vy
  }
}
