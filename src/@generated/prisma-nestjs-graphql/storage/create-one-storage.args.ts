import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageCreateInput } from './storage-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneStorageArgs {

    @Field(() => StorageCreateInput, {nullable:false})
    @Type(() => StorageCreateInput)
    data!: StorageCreateInput;
}
