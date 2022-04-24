import { Module } from '@nestjs/common';
import { StorageService } from './storage.service';
import { StorageResolver } from './storage.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, StorageResolver, StorageService]
})
export class StorageModule {}
