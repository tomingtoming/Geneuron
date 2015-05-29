package geneuron.creature

import org.newdawn.slick.Color

class Status(m: Float, v: Float, r: Float, g: Float, b: Float) {
  val color:Color = new Color(r, g, b)
  var (mode, view) = (m, v)
}
