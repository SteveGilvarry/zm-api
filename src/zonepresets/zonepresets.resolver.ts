import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ZonepresetsService } from './zonepresets.service';
import { ZonePresetsCreateInput } from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-create.input';
import { ZonePresetsUpdateInput } from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-update.input';
import { ZonePresetsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/zone-presets/zone-presets-where-unique.input';


@Resolver('Zonepreset')
export class ZonepresetsResolver {
  constructor(private readonly zonepresetsService: ZonepresetsService) {}

  @Mutation('createZonepreset')
  async create(
    @Args('createZonepresetInput') createZonepresetInput: ZonePresetsCreateInput
  ) {
    const created = await this.zonepresetsService.create(createZonepresetInput);
  }

  @Query('zonepresets')
  findAll() {
    return this.zonepresetsService.findAll();
  }

  @Query('zonepreset')
  findOne(@Args('id') Id: number) {
    return this.zonepresetsService.findOne( {Id});
  }

  @Mutation('updateZonepreset')
  update(
    @Args('id') Id: number,
    @Args('updateZonepresetInput') updateZonepresetInput: ZonePresetsUpdateInput) {
    return this.zonepresetsService.update( { Id }, updateZonepresetInput);
  }

  @Mutation('removeZonepreset')
  remove(@Args('id') Id: number) {
    return this.zonepresetsService.remove( { Id});
  }
}
