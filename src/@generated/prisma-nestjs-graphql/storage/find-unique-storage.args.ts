import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereUniqueInput } from './storage-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class FindUniqueStorageArgs {

    @Field(() => StorageWhereUniqueInput, {nullable:false})
    @Type(() => StorageWhereUniqueInput)
    where!: StorageWhereUniqueInput;
}
