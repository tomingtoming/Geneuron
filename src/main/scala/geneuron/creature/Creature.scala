package geneuron.creature

import geneuron.Coefficient
import org.newdawn.slick.geom.Circle
import org.newdawn.slick.{Color, GameContainer, Graphics}

class Creature(p: Physics, n: Neuron, s: Status) {
  var (physics, neuron, status) = (p, n, s)
  val circle: Circle = new Circle(physics.x, physics.y, 10.0F)
  def update(xs:Array[Float]) = {
    neuron.update(xs)
  }
  def render(gc: GameContainer, g: Graphics) = {
    /* Swap Context */
    val swapColor = g.getColor
    g.pushTransform()

    /* Circle */
    circle.setCenterX(physics.x)
    circle.setCenterY(physics.y)
    g.setColor(status.color)
    g.fill(circle)

    /* Line */
    g.setColor(Color.pink)
    g.translate(physics.x, physics.y)
    g.rotate(0F, 0F, physics.deg)
    g.drawLine(0F, 0F, Coefficient.sight, Coefficient.sight)
    g.resetTransform()

    /* Reset to Context */
    g.setColor(swapColor)
    g.popTransform()
  }
}
