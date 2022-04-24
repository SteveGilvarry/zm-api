import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@InputType()
export class ModelsManufacturerIdNameCompoundUniqueInput {

    @Field(() => Int, {nullable:false})
    ManufacturerId!: number;

    @Field(() => String, {nullable:false})
    Name!: string;
}
