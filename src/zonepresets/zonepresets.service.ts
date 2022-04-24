import { Injectable } from '@nestjs/common';
import { ZonePresetsCreateInput} from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-create.input';
import { ZonePresetsUpdateInput} from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-update.input';
import { ZonePresetsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ZonepresetsService {
  constructor(private prisma: PrismaService) {}

  create(createZonepresetInput: ZonePresetsCreateInput) {
    return this.prisma.zonePresets.create({
      data: createZonepresetInput
    });
  }

  findAll() {
    return this.prisma.zonePresets.findMany();
  }

  findOne(zonepresetWhereUniqueInput: ZonePresetsWhereUniqueInput) {
    return this.prisma.zonePresets.findUnique({
      where: zonepresetWhereUniqueInput
    });
  }

  update(
    zonepresetWhereUniqueInput: ZonePresetsWhereUniqueInput,
    updateZonepresetInput: ZonePresetsUpdateInput
  ) {
    return this.prisma.zonePresets.update({
      where: zonepresetWhereUniqueInput,
      data: updateZonepresetInput
    });
  }

  remove(zonepresetWhereUniqueInput: ZonePresetsWhereUniqueInput) {
    return this.prisma.zonePresets.delete({
      where: zonepresetWhereUniqueInput
    });
  }
}
