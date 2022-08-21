import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereInput } from './storage-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyStorageArgs {

    @Field(() => StorageWhereInput, {nullable:true})
    @Type(() => StorageWhereInput)
    where?: StorageWhereInput;
}
