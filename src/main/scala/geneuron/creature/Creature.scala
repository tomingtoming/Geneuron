package geneuron.creature

import geneuron.Coefficient
import geneuron.neuron.{NeuronCluster, Neuron}
import org.newdawn.slick.geom.Circle
import org.newdawn.slick.{Color, GameContainer, Graphics}
import scala.util.Random

object Creature {
  val neuronLayers = Seq(29, 30, 7)
}

/**
 * Decisions
 * 0: Viewing angle
 * 1: Power level of left propeller
 * 2: Power level of right propeller
 * 3: Body color R
 * 4: Body color G
 * 5: Body color B
 * 6: Intention(Reproduce:True, Eat:False)
 */
class Creature(val physics: Physics, val gene: Array[Double] = Array.fill(30 * 30 + 31 * 7)(Random.nextDouble())) {

  val neuron = new NeuronCluster(Creature.neuronLayers, gene)
  var viewingAngle: Float = 0.0F
  var leftProp: Float = 0.0F
  var rightProp: Float = 0.0F
  var intention: Boolean = false

  val circle: Circle = new Circle(physics.x, physics.y, 10.0F)
  val color: Color = new Color(1F, 1F, 1F)

  def process(in: Array[Float]) = {
    val out = neuron.process(in.map(_.toDouble)).map(_.toFloat)
    viewingAngle = out(0) * Coefficient.angle
    leftProp = out(1) * Coefficient.prop
    rightProp = out(2) * Coefficient.prop
    color.r = out(3) * 255.0F
    color.g = out(4) * 255.0F
    color.b = out(5) * 255.0F
    intention = 0.5F < out(6)
  }

  def render(gc: GameContainer, g: Graphics) = {
    /* Swap Context */
    val swapColor = g.getColor
    g.pushTransform()

    /* Circle */
    circle.setCenterX(physics.x)
    circle.setCenterY(physics.y)
    g.setColor(color)
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
