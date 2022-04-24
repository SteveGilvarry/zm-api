import { Module } from '@nestjs/common';
import { ManufacturersService } from './manufacturers.service';
import { ManufacturersResolver } from './manufacturers.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService,ManufacturersResolver, ManufacturersService]
})
export class ManufacturersModule {}
