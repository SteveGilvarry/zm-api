import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageCreateManyInput } from './storage-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyStorageArgs {

    @Field(() => [StorageCreateManyInput], {nullable:false})
    @Type(() => StorageCreateManyInput)
    data!: Array<StorageCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
