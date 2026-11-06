// Copyright (c) 2026 create-horizon contributors
// SPDX-License-Identifier: MIT
// Runs the original actor tick without constructing a GPU-backed instance.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.lang.reflect.Field;
import java.util.Locale;
import com.simibubi.create.content.contraptions.Contraption;
import com.simibubi.create.content.contraptions.actors.harvester.HarvesterActorVisual;
import com.simibubi.create.content.contraptions.bearing.BearingContraption;
import com.simibubi.create.content.contraptions.behaviour.MovementContext;
import com.simibubi.create.content.contraptions.render.ActorVisual;
import net.createmod.catnip.math.AngleHelper;
import net.minecraft.core.Direction;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

public class HarvesterBladeOracle {
    private static Field field(Class<?> klass, String name) throws Exception {
        Field f = klass.getDeclaredField(name);
        f.setAccessible(true);
        return f;
    }
    public static void main(String[] args) throws Exception {
        Unsafe unsafe = (Unsafe) field(Unsafe.class, "theUnsafe").get(null);
        var blade = (HarvesterActorVisual) unsafe.allocateInstance(HarvesterActorVisual.class);
        var contraption = (Contraption) unsafe.allocateInstance(BearingContraption.class);
        var context = (MovementContext) unsafe.allocateInstance(MovementContext.class);
        context.contraption = contraption;
        field(ActorVisual.class, "context").set(blade, context);
        Field facing = field(HarvesterActorVisual.class, "facing");
        Field previous = field(HarvesterActorVisual.class, "previousRotation");
        Field rotation = field(HarvesterActorVisual.class, "rotation");
        try (var reader = new BufferedReader(new InputStreamReader(System.in))) {
            for (String line; (line = reader.readLine()) != null;) {
                var a = line.split(" ");
                facing.set(blade, Direction.valueOf(a[0].toUpperCase(Locale.ROOT)));
                context.motion = new Vec3(Double.parseDouble(a[1]), Double.parseDouble(a[2]), Double.parseDouble(a[3]));
                context.relativeMotion = new Vec3(Double.parseDouble(a[4]), Double.parseDouble(a[5]), Double.parseDouble(a[6]));
                contraption.stalled = a[7].equals("1");
                context.disabled = a[8].equals("1");
                blade.tick();
                double p = previous.getDouble(blade);
                double r = rotation.getDouble(blade);
                System.out.println(p + " " + r + " " + AngleHelper.angleLerp(0.5, p, r));
            }
        }
    }
}
