import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { DevicesService } from './devices.service';
import { DevicesCreateInput} from '../@generated/prisma-nestjs-graphql/devices/devices-create.input';
import { DevicesUpdateInput} from '../@generated/prisma-nestjs-graphql/devices/devices-update.input';
import { DevicesWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/devices/devices-where-unique.input';

@Resolver('Device')
export class DevicesResolver {
  constructor(private readonly devicesService: DevicesService) {}

  @Mutation('createDevice')
  async create(
    @Args('createDeviceInput') createDeviceInput: DevicesCreateInput) {
    const created = await this.devicesService.create(createDeviceInput);
  }

  @Query('devices')
  findAll() {
    return this.devicesService.findAll();
  }

  @Query('device')
  findOne(@Args('id') Id: number) {
    return this.devicesService.findOne( { Id } );
  }

  @Mutation('updateDevice')
  update(
    @Args( 'id') Id: number,
    @Args('updateDeviceInput') updateDeviceInput: DevicesUpdateInput) {
    return this.devicesService.update( {Id}, updateDeviceInput );
  }

  @Mutation('removeDevice')
  remove(@Args('id') Id: number) {
    return this.devicesService.remove( {Id} );
  }
}
