import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';
import { ModelsManufacturerIdNameCompoundUniqueInput } from './models-manufacturer-id-name-compound-unique.input';

@InputType()
export class ModelsWhereUniqueInput {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => ModelsManufacturerIdNameCompoundUniqueInput, {nullable:true})
    ManufacturerId_Name?: ModelsManufacturerIdNameCompoundUniqueInput;
}
