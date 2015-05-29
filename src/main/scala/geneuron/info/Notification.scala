package geneuron.info

import geneuron.Geneuron
import org.newdawn.slick.{Graphics, GameContainer}

object Notification {
  def render(gc: GameContainer, g: Graphics, geneuron: Geneuron): Unit = {
    g.drawString("Creatures: " + geneuron.creatures.size, 10, 10)
  }
}
