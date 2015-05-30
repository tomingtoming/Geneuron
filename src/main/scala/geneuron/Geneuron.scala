package geneuron

import java.util.logging.{Level, Logger}

import geneuron.creature.{Creature, Physics}
import geneuron.info.Notification
import org.newdawn.slick._

import scala.collection.parallel.mutable
import scala.util.Random

object Geneuron {
  def main(args: Array[String]) {
    try {
      val appgc = new AppGameContainer(new Geneuron("Geneuron"))
      appgc.setDisplayMode(640, 480, false)
      appgc.start()
    } catch {
      case ex: SlickException => Logger.getLogger(classOf[Geneuron].getName).log(Level.SEVERE, null, ex)
    }
  }
}

class Geneuron(gamename: String) extends BasicGame(gamename) {
  val creatures: mutable.ParSet[Creature] = mutable.ParSet.empty
  val viewpoint: ViewPoint = new ViewPoint
  var terminate: Boolean = false
  def init(gc: GameContainer): Unit = {
    gc.setShowFPS(false)
    creatures ++= (1 to 10) map { _ =>
      new Creature(new Physics(Random.nextFloat(), Random.nextFloat(), Random.nextFloat(), Random.nextFloat(), Random.nextFloat(), Random.nextFloat()))
    }
  }
  def update(gc: GameContainer, i: Int): Unit = {
    val kd = gc.getInput.isKeyDown _
    val kp = gc.getInput.isKeyPressed _
    if (kd(Input.KEY_ESCAPE)) terminate = true
    if (kd(Input.KEY_Z)) viewpoint.zoom *= 1.05F
    if (kd(Input.KEY_X)) viewpoint.zoom *= 0.95F
    creatures.foreach { creature =>
      creature.physics.update()
      creature.process(Array.fill(29)(Random.nextFloat()))
    }
  }
  def render(gc: GameContainer, g: Graphics): Unit = {
    if (terminate) {
      gc.exit()
    } else {
      g.scale(viewpoint.zoom, viewpoint.zoom)
      creatures.toArray.foreach(_.render(gc, g))
      Notification.render(gc, g, this)
    }
  }
}
